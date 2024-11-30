
type DevClass = [u8; 3]

pub enum Event {
    InquiryComplete {},
    InquiryResult {
        bdaddr: BDAddr,
        pscan_rep_mode: u8,
        pscan_period_mod: u8,
        pscan_mode: u8,
        dev_class: DevClass,
        clock_offset: u16,
    },
    ConnComplete {
        status: u8,
        handle: u16,
        bdaddr: BDAddr,
        link_type: u8,
        encr_mode: u8,
    },
    ConnRequest {
        bdaddr: BDAddr,
        dev_class: DevClass
        link_type: u8,
    },
    DisconnComplete {
        status: u8,
        handle: u16,
        reason: u8,
    },
    AuthComplete {
        status: u8,
        handle: u16,
    },
    RemoteNameReqComplete {
        status: u8,
        bdaddr: BDAddr,
        name: String, // Has a maximum length HCI_MAX_NAME_LENGTH
    },
    EncryptChange {
        status: u8,
        handle: u16,
        encrypt: u8,
    },
    ChangeConnLinkKeycomplete {
        status: u8,
        handle: u16,
    },
    MasterLinkKeyComplete {
        status: u8,
        handle: u16,
        key_flag: u8,
    },
    ReadRemoteFeaturesComplete {
        status: u8,
        handle: u16,
        features: [u8; 8],
    },
    ReadRemoteVersionComplete {
        status: u8,
        handle: u16,
        lmp_ver: u8,
        manufacturer: u16,
        lmp_subver: u16,
    },
    QosSetupComplete {
        status: u8,
        handle: u16,
        flags: u8, // Reserved
        qos: HCIQos,
    },
    CmdComplete {
        ncmd: u8,
        opcode: u16,
    },
    CmdStatus {
        status: u8,
        ncmd: u8,
        opcode: u16, // Uh oh.
    },
    HardwareError {
        code: u8,
    },
    FlushOccurred {
        handle: u16,
    },
    RoleChange {
        status: u8,
        bdaddr: BDAddr,
        role: u8,
    },
    NumCompPkts {
        num_hndl: u8,
        data: // variable-length part.
    },
    ModeChange {
        status: u8,
        handle: u16,
        mode: u8,
        interval: u16,
    },
    ReturnLinkKeys {
        num_keys: u8,
        data: // variable-length part.
    },
    PinCodeReq {
        bdaddr: BDAddr,
    },
    LinkKeyReq {
        bdaddr: BDAddr,
    },
    LinkKeyNotify {
        bdaddr: BDAddr,
        link_key: [u8; 16],
        key_type: u8,
    },
    LoopbackCommand {},
    DataBufferOverflow {
        link_type: u8,
    },
    MaxSlotsChange {
        handle: u16,
        max_slots: u8,
    },
    ReadClockOffsetComplete {
        status: u8,
        handle: u16,
        clock_offset: u16,
    },
    ConnPtypeChanged {
        status: u8,
        handle: u16,
        ptype: u16,
    },
    QosViolation {
        handle: u16,
    },
    PscanRepModeChange {
        bdaddr: BDAddr,
        pscan_rep_mode: u8,
    },
    FlowSpecComplete {
        status: u8,
        handle: u16,
        flags: u8,
        direction: u8,
        qos: HciQos,
    },
    InquiryResultWithRssi {
        bdaddr: BDAddr,
        pscan_rep_mode: u8,
        pscan_period_mod: u8,
        dev_class: DevClass,
        clock_offset: u16,
        rssi: i8,
    },
    InquiryInfoWithRssi {
        bdaddr: BDAddr,
        pscan_rep_mode: u8,
        pscan_period_mode: u8,
        pscan_mode: u8,
        dev_class: DevClass,
        clock_offset: u16,
        rssi: i8,
    },
    ReadRemoteExtFeaturesComplete {
        status: u8,
        handle: u16,
        page_num: u8,
        max_page_num: u8,
        features: [u8; 8],
    },
    SyncConnComplete {
        status: u8,
        handle: u16,
        trans_interval: u8,
        retrans_window: u8,
        rx_pkt_len: u16,
        tx_pkt_len: u16,
    },
    // TODO
}

// By the way. Is it really worth it to implement all of these events when we're just looking for
// status, command complete, and timeout? I think, considering how little I know about each of
// these structs, it doesn't seem like a full HCI library is really going to manifest, at least not
// in the middle of this project.
//
// So I am in full support of implementing only the few enums that are relevant and disregarding
// the rest.


#define EVT_INQUIRY_COMPLETE		0x01

#define EVT_INQUIRY_RESULT		0x02
typedef struct {
	bdaddr_t	bdaddr;
	uint8_t		pscan_rep_mode;
	uint8_t		pscan_period_mode;
	uint8_t		pscan_mode;
	uint8_t		dev_class[3];
	uint16_t	clock_offset;
} __attribute__ ((packed)) inquiry_info;
#define INQUIRY_INFO_SIZE 14

#define EVT_CONN_COMPLETE		0x03
typedef struct {
	uint8_t		status;
	uint16_t	handle;
	bdaddr_t	bdaddr;
	uint8_t		link_type;
	uint8_t		encr_mode;
} __attribute__ ((packed)) evt_conn_complete;
#define EVT_CONN_COMPLETE_SIZE 11


pub trait WriteEvent {
    fn write_to(socket: Socket) -> usize {
    }
}


