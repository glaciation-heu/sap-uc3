use poem::Result;
use poem_openapi::{
    payload::Json,
    ApiResponse, Object, OpenApi,
};
use sysinfo::System;

pub struct SysStatusApi;
#[OpenApi]
impl SysStatusApi {
    /// Returns status code 200. Used to check if service is available.
    #[oai(path = "/ping", method = "get")]
    async fn ping(&self) -> Result<()> {
        Ok(())
    }

    /// Get system informations.
    #[oai(path = "/sys_status", method = "get")]
    async fn sys_status(&self) -> Result<SysStatusResponse> {
        sys_status()
    }
}

#[derive(Object)]
pub struct SysStatus {
    mem_consumption: f32,
    kernel_version: Option<String>,
    os_version: Option<String>,
    host_name: Option<String>,
}

#[derive(ApiResponse)]
pub enum SysStatusResponse {
    #[oai(status = 200)]
    Ok(Json<SysStatus>),
}

pub fn sys_status() -> Result<SysStatusResponse> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let status = SysStatus {
        mem_consumption: sys.used_memory() as f32 / sys.total_memory() as f32,
        kernel_version: System::kernel_version(),
        os_version: System::long_os_version(),
        host_name: System::host_name(),
    };
    Ok(SysStatusResponse::Ok(Json(status)))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_sys_status() -> Result<()> {
        let resp = sys_status()?;
        if let SysStatusResponse::Ok(Json(res)) = resp {
            assert_eq!(res.host_name, System::host_name());
            assert_eq!(res.os_version, System::long_os_version());
            assert_eq!(res.kernel_version, System::kernel_version());
        } else {
            assert!(false, "Sys status should return OK")
        }
        Ok(())
    }
}
