use crate::cgroup::{user_slice_limit_cpu, user_slice_unlimit_cpu};
use crate::input::{Input, KEY_AUX_MENU, KEY_MAIN_MENU};
use cec_rs::{
    CecCommand, CecConnection, CecConnectionCfgBuilder, CecDeviceType, CecDeviceTypeVec,
    CecKeypress, CecLogMessage, CecLogicalAddress, CecOpcode, CecUserControlCode,
};
use evdev::{AbsoluteAxisCode, KeyCode};
use log::{debug, error, info, trace};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
enum Ev {
    Key(KeyCode),
    Hat(AbsoluteAxisCode, i32),
}

fn on_key_press(keypress: CecKeypress, input: Arc<Input>) {
    trace!(
        "onKeyPress: {:?}, keycode: {:?}, duration: {:?}",
        keypress, keypress.keycode, keypress.duration
    );
    let input_key = match keypress.keycode {
        CecUserControlCode::Select => Some(Ev::Key(KeyCode::BTN_SOUTH)),
        CecUserControlCode::Up => Some(Ev::Hat(AbsoluteAxisCode::ABS_HAT0Y, -1)),
        CecUserControlCode::Down => Some(Ev::Hat(AbsoluteAxisCode::ABS_HAT0Y, 1)),
        CecUserControlCode::Left => Some(Ev::Hat(AbsoluteAxisCode::ABS_HAT0X, -1)),
        CecUserControlCode::Right => Some(Ev::Hat(AbsoluteAxisCode::ABS_HAT0X, 1)),
        CecUserControlCode::RootMenu => Some(Ev::Key(KEY_MAIN_MENU)),
        CecUserControlCode::SetupMenu => Some(Ev::Key(KEY_MAIN_MENU)),
        CecUserControlCode::ContentsMenu => Some(Ev::Key(KEY_MAIN_MENU)),
        CecUserControlCode::FavoriteMenu => Some(Ev::Key(KEY_AUX_MENU)),
        CecUserControlCode::Exit => Some(Ev::Key(KeyCode::BTN_EAST)),
        CecUserControlCode::TopMenu => Some(Ev::Key(KEY_MAIN_MENU)),
        CecUserControlCode::Enter => Some(Ev::Key(KeyCode::BTN_START)),
        CecUserControlCode::PreviousChannel => Some(Ev::Key(KEY_MAIN_MENU)),
        CecUserControlCode::DisplayInformation => Some(Ev::Key(KeyCode::BTN_START)),
        CecUserControlCode::Play => Some(Ev::Key(KeyCode::KEY_PLAYPAUSE)),
        CecUserControlCode::Stop => Some(Ev::Key(KeyCode::KEY_STOP)),
        CecUserControlCode::Pause => Some(Ev::Key(KeyCode::KEY_PLAYPAUSE)),
        CecUserControlCode::Record => Some(Ev::Key(KeyCode::KEY_RECORD)),
        CecUserControlCode::Rewind => Some(Ev::Key(KeyCode::KEY_REWIND)),
        CecUserControlCode::FastForward => Some(Ev::Key(KeyCode::KEY_FASTFORWARD)),
        CecUserControlCode::Eject => Some(Ev::Key(KeyCode::KEY_EJECTCD)),
        CecUserControlCode::Forward => Some(Ev::Key(KeyCode::KEY_FORWARD)),
        CecUserControlCode::Backward => Some(Ev::Key(KeyCode::KEY_BACK)),
        CecUserControlCode::StopRecord => Some(Ev::Key(KeyCode::KEY_STOP_RECORD)),
        CecUserControlCode::PauseRecord => Some(Ev::Key(KeyCode::KEY_PAUSE_RECORD)),
        CecUserControlCode::VideoOnDemand => Some(Ev::Key(KEY_MAIN_MENU)),
        CecUserControlCode::ElectronicProgramGuide => Some(Ev::Key(KEY_MAIN_MENU)),
        CecUserControlCode::SelectMediaFunction => Some(Ev::Key(KeyCode::BTN_START)),
        CecUserControlCode::F1Blue => Some(Ev::Key(KeyCode::BTN_C)),
        CecUserControlCode::F2Red => Some(Ev::Key(KeyCode::BTN_SOUTH)),
        CecUserControlCode::F3Green => Some(Ev::Key(KeyCode::BTN_EAST)),
        CecUserControlCode::F4Yellow => Some(Ev::Key(KeyCode::BTN_NORTH)),
        CecUserControlCode::AnReturn => Some(Ev::Key(KEY_AUX_MENU)),
        CecUserControlCode::AnChannelsList => Some(Ev::Key(KEY_AUX_MENU)),
        _ => None,
    };

    // If the duration is 0 it's a down event, if it is not zero it is an up event.
    if let Some(input_ev) = input_key {
        if keypress.duration.is_zero() {
            match input_ev {
                Ev::Key(input_key) => match input.press(input_key) {
                    Ok(_) => debug!("pressed down {:?}", input_key),
                    Err(err) => error!("failed pressing {input_key:?}: {err:?}"),
                },
                Ev::Hat(hat, pos) => match input.axis(hat, pos) {
                    Ok(_) => debug!("pressed axis {:?}", (hat, pos)),
                    Err(err) => error!("failed pressing axis {:?}: {:?}", (hat, pos), err),
                },
            }
        } else {
            match input_ev {
                Ev::Key(input_key) => match input.release(input_key) {
                    Ok(_) => debug!("released {:?}", input_key),
                    Err(err) => error!("failed pressing {input_key:?}: {err:?}"),
                },
                Ev::Hat(hat, _) => match input.axis(hat, 0) {
                    Ok(_) => debug!("released axis {:?}", (hat, 0)),
                    Err(err) => error!("failed released axis {:?}: {:?}", (hat, 0), err),
                },
            }
        }
    }
}

fn on_command_received(command: CecCommand, input: Arc<Input>) {
    trace!(
        "onCommandReceived:  opcode: {:?}, initiator: {:?}",
        command.opcode, command.initiator
    );

    match command.opcode {
        CecOpcode::RequestActiveSource => {
            if command.initiator == CecLogicalAddress::Tv {
                info!("TV came online, unrestricting CPU");
                user_slice_unlimit_cpu().ok();
            }
        }
        CecOpcode::Standby => {
            if command.initiator == CecLogicalAddress::Tv {
                info!("TV came offline, restricting CPU");
                user_slice_limit_cpu().ok();
            }
        }
        CecOpcode::Play => {
            let input_key = KeyCode::KEY_PLAYPAUSE;
            let duration = Duration::from_millis(16);
            match input.press_and_release(input_key, duration) {
                Ok(_) => debug!("pressed {:?} for {:?}", input_key, duration),
                Err(err) => error!("failed pressing {input_key:?}: {err:?}"),
            }
        }
        _ => {}
    }
}

fn on_log_message(log_message: CecLogMessage) {
    trace!(
        "logMessageRecieved:  time: {}, level: {}, message: {}",
        log_message.time.as_secs(),
        log_message.level,
        log_message.message
    );
}

pub fn run_cec(input: Input) -> CecConnection {
    let input = Arc::new(input);
    let input2 = input.clone();
    let cfg = CecConnectionCfgBuilder::default()
        .activate_source(true)
        .device_name(hostname::get().unwrap().to_string_lossy().to_string())
        .key_press_callback(Box::new(move |p| on_key_press(p, input.clone())))
        .command_received_callback(Box::new(move |p| on_command_received(p, input2.clone())))
        .log_message_callback(Box::new(on_log_message))
        .device_types(CecDeviceTypeVec::new(CecDeviceType::RecordingDevice))
        .build()
        .unwrap();

    cfg.open().unwrap()
}
