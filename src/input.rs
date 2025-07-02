use evdev::uinput::VirtualDevice;
use evdev::{
    AbsInfo, AbsoluteAxisCode, AbsoluteAxisEvent, AttributeSet, BusType, EventType, InputEvent,
    InputId, KeyCode, KeyEvent, MiscCode, UinputAbsSetup,
};
use std::io;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

pub const KEY_MAIN_MENU: KeyCode = KeyCode::KEY_MENU;
pub const KEY_AUX_MENU: KeyCode = KeyCode::KEY_CONTEXT_MENU;

pub struct Input {
    vd: Mutex<VirtualDevice>,
}

impl Input {
    pub fn press_and_release(&self, key: KeyCode, duration: Duration) -> io::Result<()> {
        let mut vd = self.vd.lock().unwrap();
        vd.emit(&[*KeyEvent::new(key, 1)])?;
        sleep(duration);
        vd.emit(&[InputEvent::new(EventType::KEY.0, key.0, 0)])
    }
    pub fn press(&self, key: KeyCode) -> io::Result<()> {
        let mut vd = self.vd.lock().unwrap();

        vd.emit(&[*KeyEvent::new(key, 1)])
    }
    pub fn release(&self, key: KeyCode) -> io::Result<()> {
        let mut vd = self.vd.lock().unwrap();

        vd.emit(&[InputEvent::new(EventType::KEY.0, key.0, 0)])
    }
    pub fn axis(&self, hat: AbsoluteAxisCode, pos: i32) -> io::Result<()> {
        let mut vd = self.vd.lock().unwrap();
        vd.emit(&[*AbsoluteAxisEvent::new(hat, pos)])
    }
}

pub fn setup_input() -> Input {
    let mut keys = AttributeSet::<KeyCode>::new();
    keys.insert(KeyCode::BTN_SOUTH);
    keys.insert(KeyCode::BTN_NORTH);
    keys.insert(KeyCode::BTN_WEST);
    keys.insert(KeyCode::BTN_EAST);
    keys.insert(KeyCode::BTN_TL);
    keys.insert(KeyCode::BTN_TR);
    keys.insert(KeyCode::BTN_TL2);
    keys.insert(KeyCode::BTN_TR2);
    keys.insert(KeyCode::BTN_START);
    keys.insert(KeyCode::BTN_SELECT);
    keys.insert(KeyCode::BTN_Z);
    keys.insert(KeyCode::BTN_C);
    keys.insert(KEY_MAIN_MENU);
    keys.insert(KEY_AUX_MENU);
    keys.insert(KeyCode::KEY_PLAYPAUSE);
    keys.insert(KeyCode::KEY_STOP);
    keys.insert(KeyCode::KEY_RECORD);
    keys.insert(KeyCode::KEY_REWIND);
    keys.insert(KeyCode::KEY_FASTFORWARD);
    keys.insert(KeyCode::KEY_EJECTCD);
    keys.insert(KeyCode::KEY_FORWARD);
    keys.insert(KeyCode::KEY_BACK);
    keys.insert(KeyCode::KEY_STOP_RECORD);
    keys.insert(KeyCode::KEY_PAUSE_RECORD);

    let mut mscs = AttributeSet::<MiscCode>::new();
    mscs.insert(MiscCode::MSC_SCAN);

    Input {
        vd: Mutex::new(
            VirtualDevice::builder()
                .unwrap()
                .name("CEC")
                // Mock an 8bit do controller
                .input_id(InputId::new(BusType::BUS_BLUETOOTH, 0x45e, 0x2e0, 0x903))
                .with_keys(&keys)
                .unwrap()
                .with_msc(&mscs)
                .unwrap()
                .with_absolute_axis(&UinputAbsSetup::new(
                    AbsoluteAxisCode::ABS_HAT0X,
                    AbsInfo::new(0, -1, 1, 0, 0, 0),
                ))
                .unwrap()
                .with_absolute_axis(&UinputAbsSetup::new(
                    AbsoluteAxisCode::ABS_HAT0Y,
                    AbsInfo::new(0, -1, 1, 0, 0, 0),
                ))
                .unwrap()
                .build()
                .unwrap(),
        ),
    }
}
