use crate::error::ParseErrorType;
use crate::unit::{BaseUnit, Unit};
use crate::{
    error::ParseError,
    unit::{service::ServiceUnitAttr, BaseUnitAttr, InstallUnitAttr, UnitType},
};

#[cfg(target_os = "dragonos")]
use drstd as std;

use hashbrown::HashMap;
use lazy_static::lazy_static;
use std::format;
use std::fs::File;
use std::io::{self, BufRead};
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;
use std::string::ToString;

pub mod parse_target;
pub mod parse_service;
pub mod parse_util;

//对应Unit段类型
#[derive(PartialEq, Clone, Copy)]
pub enum Segment {
    None,
    Unit,
    Install,
    Service,
}

lazy_static! {
    pub static ref UNIT_SUFFIX: HashMap<&'static str, UnitType> = {
        let mut table = HashMap::new();
        table.insert("automount", UnitType::Automount);
        table.insert("device", UnitType::Device);
        table.insert("mount", UnitType::Mount);
        table.insert("path", UnitType::Path);
        table.insert("scope", UnitType::Scope);
        table.insert("service", UnitType::Service);
        table.insert("slice", UnitType::Automount);
        table.insert("automount", UnitType::Slice);
        table.insert("socket", UnitType::Socket);
        table.insert("swap", UnitType::Swap);
        table.insert("target", UnitType::Target);
        table.insert("timer", UnitType::Timer);
        table
    };
    pub static ref SEGMENT_TABLE: HashMap<&'static str, Segment> = {
        let mut table = HashMap::new();
        table.insert("[Unit]", Segment::Unit);
        table.insert("[Install]", Segment::Install);
        table.insert("[Service]", Segment::Service);
        table
    };
    pub static ref INSTALL_UNIT_ATTR_TABLE: HashMap<&'static str, InstallUnitAttr> = {
        let mut unit_attr_table = HashMap::new();
        unit_attr_table.insert("WantedBy", InstallUnitAttr::WantedBy);
        unit_attr_table.insert("RequiredBy", InstallUnitAttr::RequiredBy);
        unit_attr_table.insert("Also", InstallUnitAttr::Also);
        unit_attr_table.insert("Alias", InstallUnitAttr::Alias);
        unit_attr_table
    };
    pub static ref SERVICE_UNIT_ATTR_TABLE: HashMap<&'static str, ServiceUnitAttr> = {
        let mut unit_attr_table = HashMap::new();
        unit_attr_table.insert("Type", ServiceUnitAttr::Type);
        unit_attr_table.insert("RemainAfterExit", ServiceUnitAttr::RemainAfterExit);
        unit_attr_table.insert("ExecStart", ServiceUnitAttr::ExecStart);
        unit_attr_table.insert("ExecStartPre", ServiceUnitAttr::ExecStartPre);
        unit_attr_table.insert("ExecStartPos", ServiceUnitAttr::ExecStartPos);
        unit_attr_table.insert("ExecReload", ServiceUnitAttr::ExecReload);
        unit_attr_table.insert("ExecStop", ServiceUnitAttr::ExecStop);
        unit_attr_table.insert("ExecStopPost", ServiceUnitAttr::ExecStopPost);
        unit_attr_table.insert("RestartSec", ServiceUnitAttr::RestartSec);
        unit_attr_table.insert("Restart", ServiceUnitAttr::Restart);
        unit_attr_table.insert("TimeoutStartSec", ServiceUnitAttr::TimeoutStartSec);
        unit_attr_table.insert("TimeoutStopSec", ServiceUnitAttr::TimeoutStopSec);
        unit_attr_table.insert("Environment", ServiceUnitAttr::Environment);
        unit_attr_table.insert("EnvironmentFile", ServiceUnitAttr::EnvironmentFile);
        unit_attr_table.insert("Nice", ServiceUnitAttr::Nice);
        unit_attr_table.insert("WorkingDirectory", ServiceUnitAttr::WorkingDirectory);
        unit_attr_table.insert("RootDirectory", ServiceUnitAttr::RootDirectory);
        unit_attr_table.insert("User", ServiceUnitAttr::User);
        unit_attr_table.insert("Group", ServiceUnitAttr::Group);
        unit_attr_table.insert("MountFlags", ServiceUnitAttr::MountFlags);
        unit_attr_table
    };
    pub static ref BASE_UNIT_ATTR_TABLE: HashMap<&'static str, BaseUnitAttr> = {
        let mut unit_attr_table = HashMap::new();
        unit_attr_table.insert("Description", BaseUnitAttr::Description);
        unit_attr_table.insert("Documentation", BaseUnitAttr::Documentation);
        unit_attr_table.insert("Requires", BaseUnitAttr::Requires);
        unit_attr_table.insert("Wants", BaseUnitAttr::Wants);
        unit_attr_table.insert("After", BaseUnitAttr::After);
        unit_attr_table.insert("Before", BaseUnitAttr::Before);
        unit_attr_table.insert("Binds To", BaseUnitAttr::BindsTo);
        unit_attr_table.insert("Part Of", BaseUnitAttr::PartOf);
        unit_attr_table.insert("OnFailure", BaseUnitAttr::OnFailure);
        unit_attr_table.insert("Conflicts", BaseUnitAttr::Conflicts);
        unit_attr_table
    };
    pub static ref BASE_IEC: HashMap<&'static str, u64> = {
        let mut table = HashMap::new();
        table.insert(
            "E",
            1024u64 * 1024u64 * 1024u64 * 1024u64 * 1024u64 * 1024u64,
        );
        table.insert("P", 1024u64 * 1024u64 * 1024u64 * 1024u64 * 1024u64);
        table.insert("T", 1024u64 * 1024u64 * 1024u64 * 1024u64);
        table.insert("G", 1024u64 * 1024u64 * 1024u64);
        table.insert("M", 1024u64 * 1024u64);
        table.insert("K", 1024u64);
        table.insert("B", 1u64);
        table.insert("", 1u64);
        table
    };
    pub static ref BASE_SI: HashMap<&'static str, u64> = {
        let mut table = HashMap::new();
        table.insert(
            "E",
            1000u64 * 1000u64 * 1000u64 * 1000u64 * 1000u64 * 1000u64,
        );
        table.insert("P", 1000u64 * 1000u64 * 1000u64 * 1000u64 * 1000u64);
        table.insert("T", 1000u64 * 1000u64 * 1000u64 * 1000u64);
        table.insert("G", 1000u64 * 1000u64 * 1000u64);
        table.insert("M", 1000u64 * 1000u64);
        table.insert("K", 1000u64);
        table.insert("B", 1u64);
        table.insert("", 1u64);
        table
    };
    pub static ref SEC_UNIT_TABLE: HashMap<&'static str, u64> = {
        let mut table = HashMap::new();
        table.insert("h", 60 * 60 * 1000 * 1000 * 1000);
        table.insert("min", 60 * 1000 * 1000 * 1000);
        table.insert("m", 60 * 1000 * 1000 * 1000);
        table.insert("s", 1000 * 1000 * 1000);
        table.insert("", 1000 * 1000 * 1000);
        table.insert("ms", 1000 * 1000);
        table.insert("us", 1000);
        table.insert("ns", 1);
        table
    };
}

//用于解析Unit共有段的方法
pub struct UnitParser;

impl UnitParser {
    /// @brief 从path获取到BufReader,此方法将会检验文件类型
    ///
    /// 从path获取到BufReader,此方法将会检验文件类型
    ///
    /// @param path 需解析的文件路径
    ///
    /// @param unit_type 指定Unit类型
    ///
    /// @return 成功则返回对应BufReader，否则返回Err
    fn get_unit_reader(path: &str, unit_type: UnitType) -> Result<io::BufReader<File>, ParseError> {
        let suffix = match path.rfind('.') {
            Some(idx) => &path[idx + 1..],
            None => {
                return Err(ParseError::new(ParseErrorType::EFILE, path.to_string(),0));
            }
        };
        let u_type = UNIT_SUFFIX.get(suffix);
        if u_type.is_none() {
            return Err(ParseError::new(ParseErrorType::EFILE, path.to_string(),0));
        }
        if *(u_type.unwrap()) != unit_type {
            return Err(ParseError::new(ParseErrorType::EFILE, path.to_string(),0));
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => {
                return Err(ParseError::new(ParseErrorType::EFILE, path.to_string(),0));
            }
        };
        return Ok(io::BufReader::new(file));
    }

    /// @brief 将path路径的文件解析为unit_type类型的Unit
    ///
    /// 该方法解析每个Unit共有的段(Unit,Install),其余独有的段属性将会交付T类型的Unit去解析
    ///
    /// @param path 需解析的文件路径
    ///
    /// @param unit_type 指定Unit类型
    ///
    /// @return 解析成功则返回Ok(Rc<T>)，否则返回Err
    pub fn parse<T: Unit + Default>(path: &str, unit_type: UnitType) -> Result<Rc<T>, ParseError> {
        let mut unit: T = T::default();
        let mut unit_base = BaseUnit::default();
        //设置unit类型标记
        unit_base.set_unit_type(unit_type);

        let reader = UnitParser::get_unit_reader(path, unit_type)?;

        //用于记录当前段的类型
        let mut segment = Segment::None;
        //用于处理多行对应一个属性的情况
        let mut last_attr = ServiceUnitAttr::None;

        //一行一行向下解析
        let lines = reader
            .lines()
            .map(|line| line.unwrap())
            .collect::<Vec<String>>();
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            //空行跳过
            if line.chars().all(char::is_whitespace) {
                i += 1;
                continue;
            }
            //注释跳过
            if line.starts_with('#') {
                i += 1;
                continue;
            }
            let mut line = line.trim();
            let segment_flag = SEGMENT_TABLE.get(&line);
            if !segment_flag.is_none() {
                //如果当前行匹配到的为段名，则切换段类型继续匹配下一行
                segment = *segment_flag.unwrap();
                i += 1;
                continue;
            }
            if segment == Segment::None {
                //未找到段名则不能继续匹配
                return Err(ParseError::new(ParseErrorType::ESyntaxError, path.to_string(),i + 1));
            }

            //下面进行属性匹配
            //合并多行为一个属性的情况
            //最后一个字符为\，代表换行，将多行转换为一行统一解析
            if lines[i].ends_with('\\') {
                let mut templine = String::new();
                while lines[i].ends_with('\\') {
                    let temp = &lines[i][..lines[i].len() - 1];
                    templine = format!("{} {}", templine, temp);
                    i += 1;
                }
                templine = format!("{} {}", templine, lines[i]);
                line = templine.as_str();
                i += 1;
                break;
            }
            //=号分割后第一个元素为属性，后面的均为值，若一行出现两个等号则是语法错误
            let (attr_str,val_str) = match line.find('=') {
                Some(idx) => {
                    (line[..idx].trim(), line[idx+1..].trim())
                }
                None => {
                    return Err(ParseError::new(ParseErrorType::ESyntaxError, path.to_string(),i + 1));
                }
            };
            //首先匹配所有unit文件都有的unit段和install段
            if BASE_UNIT_ATTR_TABLE.get(attr_str).is_some() {
                if segment != Segment::Unit {
                    return Err(ParseError::new(ParseErrorType::EINVAL, path.to_string(),i + 1));
                }
                if let Err(e) = unit_base.set_unit_part_attr(
                    BASE_UNIT_ATTR_TABLE.get(attr_str).unwrap(),
                    val_str,
                ){
                    let mut e = e.clone();
                    e.set_file(path);
                    e.set_linenum(i + 1);
                    return Err(e);
                }
            } else if INSTALL_UNIT_ATTR_TABLE.get(attr_str).is_some() {
                if segment != Segment::Install {
                    return Err(ParseError::new(ParseErrorType::EINVAL, path.to_string(),i + 1));
                }
                if let Err(e) = unit_base.set_install_part_attr(
                    INSTALL_UNIT_ATTR_TABLE.get(attr_str).unwrap(),
                    val_str,
                ){
                    let mut e = e.clone();
                    e.set_file(path);
                    e.set_linenum(i + 1);
                    return Err(e);
                }
            } else {
                if let Err(e) = unit.set_attr(segment, attr_str, val_str){
                    let mut e = e.clone();
                    e.set_file(path);
                    e.set_linenum(i + 1);
                    return Err(e);
                }
            }
            i += 1;
        }
        unit.set_unit_base(unit_base);
        return Ok(Rc::new(unit));
    }
}
