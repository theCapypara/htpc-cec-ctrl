use log::error;
use std::process::Command;
use tokio::io;

pub fn user_slice_limit_cpu() -> io::Result<()> {
    let r = do_user_slice_cpu(true);
    if let Err(e) = &r {
        error!("Failed limiting CPU: {}", e);
    }
    r
}

pub fn user_slice_unlimit_cpu() -> io::Result<()> {
    let r = do_user_slice_cpu(false);
    if let Err(e) = &r {
        error!("Failed unlimiting CPU: {}", e);
    }
    r
}

fn do_user_slice_cpu(restrict: bool) -> io::Result<()> {
    let value = match restrict {
        true => "10%",
        false => "",
    };
    let ok = Command::new("systemctl")
        .arg("set-property")
        .arg("--runtime")
        .arg("user-1000.slice")
        .arg(format!("CPUQuota={value}"))
        .status()?
        .success();

    if !ok {
        Err(io::Error::other("systemctl exited abnormally"))
    } else {
        Ok(())
    }
}
