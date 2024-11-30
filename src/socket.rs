use libc::{AF_BLUETOOTH, c_int, c_ushort, c_void, EAGAIN, EINTR, EIO, ETIMEDOUT, poll, pollfd, POLLIN, sa_family_t, sockaddr_storage, socklen_t, SOCK_CLOEXEC, SOCK_RAW};
use std::io::{Error, IoSlice, Read, Result, Write};
use std::ptr::{addr_of, addr_of_mut, copy_nonoverlapping};
use std::mem::{MaybeUninit, zeroed};
use std::os::fd::AsRawFd;
use socket2::{Domain, Protocol, Socket as Socket2, SockAddr, Type};

use super::filter::HciFilter;
use super::io::{ReadAs, ReadFrom, WriteAs, WriteTo};

const SOL_HCI: c_int = 0;
const HCI_FILTER: c_int = 2;
const HCI_MAX_EVENT_SIZE: usize = 260;

const HCI_COMMAND_PKT: u8 = 0x01;
const HCI_EVENT_PKT: u8 = 0x04;

const EVT_CMD_COMPLETE: u8 = 0x0E;
const EVT_CMD_STATUS: u8 = 0x0F;

const PROTO_HCI: c_int = 1;


/// Helper macro to execute a system call that returns an `io::Result`.
/// Copied from socket2.
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}



#[repr(C)]
struct HCIAddr {
    family: sa_family_t,
    device: c_ushort,
    channel: c_ushort,
}

impl HCIAddr {
    pub fn as_sock_addr(&self) -> SockAddr {
        unsafe {
            // Create a sockaddr_storage to hold the bytes of the socket address.
            let mut storage: sockaddr_storage = zeroed();
            // Copy bytes from self into the storage.
            copy_nonoverlapping(self, &mut storage as *mut _ as *mut Self, 1);
            // Create new SockAddr with a specified size.
            let len: u32 = size_of::<Self>().try_into().unwrap();

            SockAddr::new(storage, len)
        }
    }
}


/// HCI Socket
pub struct Socket(Socket2);


impl Socket {
    pub fn new(device_id: u16) -> Result<Socket> {
        let hci_domain = Domain::from(AF_BLUETOOTH);
        let hci_type = Type::from(SOCK_RAW | SOCK_CLOEXEC);
        let hci_protocol = Protocol::from(PROTO_HCI);
    
    	let socket = Socket2::new(hci_domain, hci_type, Some(hci_protocol))?;
        
        // If a device id is specified, 
        let address = HCIAddr {
            family: AF_BLUETOOTH as sa_family_t,
            device: device_id,
            channel: 0u16,
        };
        
        socket.bind(&address.as_sock_addr())?;
        
        Ok(Socket(socket))
    }

    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        self.0.send(buf)
    }
    pub fn send_vectored(&self, bufs: &[IoSlice<'_>]) -> Result<usize> {
        self.0.send_vectored(bufs)
    }

    pub fn recv(&self, buf: &mut [MaybeUninit<u8>]) -> Result<usize> {
        self.0.recv(buf)
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> c_int {
        self.0.as_raw_fd()
    }
}

impl <'a> Read for &'a mut Socket {
     fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
         self.0.read(buf)
    }
}


impl Socket {
    pub fn get_filter(&self) -> Result<HciFilter> {
        let mut filter = HciFilter::default();
        let mut filter_size = size_of::<HciFilter>() as socklen_t;

    	syscall!(getsockopt(
            self.0.as_raw_fd(),
            SOL_HCI,
            HCI_FILTER,
            addr_of_mut!(filter) as *mut c_void,
            &mut filter_size))
            .map(|_| filter)
    }

    pub fn set_filter(&self, filter: &HciFilter) -> Result<()> {
        let filter_size = size_of::<HciFilter>() as socklen_t;

    	syscall!(setsockopt(
            self.0.as_raw_fd(),
            SOL_HCI,
            HCI_FILTER,
            addr_of!(filter).cast(),
            filter_size))
            .map(|_| ())
    }
}

// Receiving events


#[derive(Copy, Clone, Debug, Default)]
struct EventHeader {
    event: u8,
    _plen: u8,
}

impl ReadFrom for EventHeader {
    fn read_from<R: Read>(mut r: R) -> Result<(Self, usize)> {
        let (event, event_size) = r.read_as::<u8>()?;
        let (plen, plen_size) = r.read_as::<u8>()?;
        Ok((EventHeader { event, _plen: plen }, event_size + plen_size))
    }
}


// We only implement these two basic events.
#[derive(Copy, Clone, Debug)]
enum EventBody {
    Unsupported,
    CmdComplete {
        _ncmd: u8,
        opcode: u16,
    },
    CmdStatus {
        _status: u8,
        _ncmd: u8,
        opcode: u16,
    },
}

impl EventBody {
    /// Return the numeric code for an event
    pub fn _code(&self) -> u8 {
        match self {
            EventBody::Unsupported { .. } => 0x00,
            EventBody::CmdComplete { .. } => 0x0E,
            EventBody::CmdStatus { .. } => 0x0F,
        }
    }
}

#[derive(Clone, Debug)]
struct Event {
    _header: EventHeader,
    body: EventBody,
    data: Box<[u8]>,
}

impl Event {
    /// Create a new event from a header and body.
    pub fn new(header: EventHeader, body: EventBody, data: Box<[u8]>) -> Self {
        Event { _header: header, body, data }
    }
}

impl ReadFrom for Event {

    /// Receive an event from a socket.
    fn read_from<R: Read>(mut r: R) -> Result<(Self, usize)> {
        let mut buf = [0u8; HCI_MAX_EVENT_SIZE];

        // Read segmented data for an entire event.
        // TODO: recv can be interrupted by EAGAIN and EINTR. We should retry in these scenarios.
        let size = r.read(&mut buf)?;

        // Get slice of buffer
        let mut buf_r: &[u8] = &buf[..size];
        
        // Skip the first byte. Not sure what it is.
        buf_r = &buf_r[1..];
        
        // Read the header.
        let (header, _) = buf_r.read_as::<EventHeader>()?;

        // Parse event body from stream
        match header.event {
            0x0E => {
                let (ncmd, _) = buf_r.read_as::<u8>()?;
                let (opcode, _) = buf_r.read_as::<u16>()?;
                Ok((Event::new(
                            header,
                            EventBody::CmdComplete { _ncmd: ncmd, opcode },
                            buf_r.into()),
                            size))
            },
            0x0F => {
                let (status, _) = buf_r.read_as::<u8>()?;
                let (ncmd, _) = buf_r.read_as::<u8>()?;
                let (opcode, _) = buf_r.read_as::<u16>()?;
                Ok((Event::new(
                            header,
                            EventBody::CmdStatus { _status: status, _ncmd: ncmd, opcode },
                            buf_r.into()),
                            size))
            },
            _ => Ok((Event::new(
                            header,
                            EventBody::Unsupported,
                            buf_r.into()),
                            size))
        }
    }
}






// hci_send_cmd(int dd, uint16_t ogf, uint16_t ocf, uint8_t plen, void *param);

/// Header for a command. Should be sent before any commands.
struct CommandHeader {
	opcode: u16,
    _plen: u8,
}
impl WriteTo for &CommandHeader {
    fn write_to<W: Write>(self, w: &mut W) -> Result<usize> {
        let opcode_size = w.write_as(self.opcode)?;
        let plen_size = w.write_as(self._plen)?;
        Ok(opcode_size + plen_size)
    }
}

impl Socket {
    pub fn send_cmd(&self, ogf: u16, ocf: u16, param: &[u8]) -> Result<usize> {
        let cmd_type = [HCI_COMMAND_PKT];
        let cmd_hdr = CommandHeader {
            opcode: cmd_opcode_pack(ogf, ocf).to_le(),
            _plen: param.len().try_into().unwrap(),
        }.bytes()?;
        // Convert cmd_hdr to a byte slice

        let mut bufs = vec![
            IoSlice::new(&cmd_type),
            IoSlice::new(&cmd_hdr),
        ];

        if param.len() > 0 {
            bufs.push(IoSlice::new(param));
        }

        self.send_vectored(bufs.as_slice())
    }
}



// hci_send_req(int dd, struct hci_request *req, int timeout);

/// Construct an opcode from ogf and ocf.
fn cmd_opcode_pack(ogf: u16, ocf: u16) -> u16 {
    (ocf & 0x03ff) | (ogf << 10)
}

/// Returns whether the socket is ready to use.
fn poll_with_timeout(socket: &Socket, timeout: c_int) -> Result<()> {
    let mut n: c_int;

    let mut p = pollfd {
        fd: socket.as_raw_fd(),
        events: POLLIN,
        revents: 0,
    };

    while unsafe {n = poll(&mut p, 1, timeout); n} < 0 {
        let e = Error::last_os_error();
        match e.raw_os_error().unwrap() {
            EAGAIN | EINTR => (), // Read again
            _ => {
                return Err(e);
            }
        }
    }

    if n == 0 {
        // Socket is not ready for reading.
        Err(Error::from_raw_os_error(ETIMEDOUT)) // Timed out
    } else {
        // Socket is ready for reading.
        Ok(())
    }
}

impl Socket {
    pub fn send_req(&mut self, ogf: u16, ocf: u16, _event: c_int, command: &[u8], mut timeout: c_int) -> Result<Box<[u8]>> {
        let mut size = 0;
        let opcode: u16 = cmd_opcode_pack(ogf, ocf).to_le();

        // Get old filter
        let old_filter = self.get_filter()?;

        // Set a new filter to catch CMD_STATUS and CMD_COMPLETE.
        let mut new_filter = HciFilter::default();
        new_filter.set_type(HCI_EVENT_PKT)?;
        new_filter.set_event(EVT_CMD_STATUS)?;
        new_filter.set_event(EVT_CMD_COMPLETE)?;
	    new_filter.set_opcode(opcode); // opcode?
	    self.set_filter(&new_filter)?;

        // Send the command through the socket.
        size += self.send_cmd(ogf, ocf, command)?;

        // Wait for a result after 10 polls.
        let result = (|mut s: &mut Socket| {
            for _ in 0..10 {

                // Poll with timeout
                if timeout > 0 {
                    // Poll repeatedly
                    poll_with_timeout(s, timeout)?;

                    timeout -= 10;
                    if timeout < 0 {
                        timeout = 0;
                    }
                }


                let (response, response_size) = s.read_as::<Event>()?;
                size += response_size;
                // When receiving, match based on response.header.event
                match response.body {
                    EventBody::CmdStatus{ _status: _, _ncmd: _, opcode: r_opcode }
                    if r_opcode == opcode => {
                        // Ignore the case in which the event listens for Event::CMD_STATUS.
                        return Err(Error::from_raw_os_error(EIO));
                    },
                    EventBody::CmdComplete{ _ncmd: _, opcode: r_opcode } if r_opcode == opcode => {
                        return Ok(response.data);
                    }
                    _ => (),
                }
            }
            Err(Error::from_raw_os_error(ETIMEDOUT))
        })(self);
        
        // Restore old filter.
	    self.set_filter(&old_filter)?;
        
        // Return result from waiting.
        result
    }
}


// After sending an HCI command, we receive a stream of HCI events, which might be responses.
// Some responses, like EVT_CMD_COMPLETE might contain other events.
// Rust's languages features provide a high level of compile-time polymorphism, but we want to
// build something with run-time polymorphism. Or, more generally, we want to parse a sequence of
// bytes into a structured sequence of tokens, just as a compiler might.
// To do this, we need internal representations of types. At worst, we have to design an entire
// tagging system from scratch. So if it feels redundant to be writing Rust code just to interpret
// bytes as types, it's probably a necessaray part of the process.

const OGF_HOST_CTL: u16 = 0x03;
const OCF_WRITE_CLASS_OF_DEV: u16 = 0x0024;

impl Socket {
    pub fn write_class_of_dev(&mut self, class: u32, timeout: c_int) -> Result<()> {
        self.send_req(OGF_HOST_CTL, OCF_WRITE_CLASS_OF_DEV,
            0, // Unused
            &class.to_le_bytes()[..3],
            timeout,
        ).map(|_| ())
    }
}


