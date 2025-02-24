use super::UnitParser;
use crate::error::ParseError;
use crate::unit::service::ServiceUnit;

#[cfg(target_os = "dragonos")]
use drstd as std;

use std::rc::Rc;
pub struct ServiceParser;

impl ServiceParser {
    /// @brief 解析Service类型Unit的
    ///
    /// 从path解析Service类型Unit
    ///
    /// @param path 需解析的文件路径
    ///
    /// @return 成功则返回Ok(Rc<ServiceUnit>)，否则返回Err
    pub fn parse(path: &str) -> Result<Rc<ServiceUnit>, ParseError> {
        //交付总解析器
        let service = UnitParser::parse::<ServiceUnit>(path, crate::unit::UnitType::Service)?;
        return Ok(service);
    }
}
