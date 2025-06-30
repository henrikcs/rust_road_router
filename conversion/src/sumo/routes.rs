/// This file has been automatically generated using xsd-parser: https://github.com/Bergmann89/xsd-parser
/// Note that some parts have been adapted where "=", "-", and values like "true" and "True" in the same enum were problematic.
///
use core::ops::{Deref, DerefMut};
use serde_derive::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct ArrivalLaneType {
    #[serde(rename = "#text")]
    pub value: ArrivalLaneTypeValue,
}
impl From<ArrivalLaneTypeValue> for ArrivalLaneType {
    fn from(value: ArrivalLaneTypeValue) -> Self {
        Self { value }
    }
}
impl From<ArrivalLaneType> for ArrivalLaneTypeValue {
    fn from(value: ArrivalLaneType) -> Self {
        value.value
    }
}
impl Deref for ArrivalLaneType {
    type Target = ArrivalLaneTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for ArrivalLaneType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ArrivalLaneTypeValue {
    #[serde(rename = "BigUint")]
    BigUint(u32),
    #[serde(rename = "current")]
    Current,
    #[serde(rename = "first")]
    First,
    #[serde(rename = "random")]
    Random,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ArrivalPosLatType {
    #[serde(rename = "#text")]
    pub value: ArrivalPosLatTypeValue,
}
impl From<ArrivalPosLatTypeValue> for ArrivalPosLatType {
    fn from(value: ArrivalPosLatTypeValue) -> Self {
        Self { value }
    }
}
impl From<ArrivalPosLatType> for ArrivalPosLatTypeValue {
    fn from(value: ArrivalPosLatType) -> Self {
        value.value
    }
}
impl Deref for ArrivalPosLatType {
    type Target = ArrivalPosLatTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for ArrivalPosLatType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ArrivalPosLatTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "left")]
    Left,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ArrivalPosType {
    #[serde(rename = "#text")]
    pub value: ArrivalPosTypeValue,
}
impl From<ArrivalPosTypeValue> for ArrivalPosType {
    fn from(value: ArrivalPosTypeValue) -> Self {
        Self { value }
    }
}
impl From<ArrivalPosType> for ArrivalPosTypeValue {
    fn from(value: ArrivalPosType) -> Self {
        value.value
    }
}
impl Deref for ArrivalPosType {
    type Target = ArrivalPosTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for ArrivalPosType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ArrivalPosTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "max")]
    Max,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ArrivalSpeedType {
    #[serde(rename = "#text")]
    pub value: ArrivalSpeedTypeValue,
}
impl From<ArrivalSpeedTypeValue> for ArrivalSpeedType {
    fn from(value: ArrivalSpeedTypeValue) -> Self {
        Self { value }
    }
}
impl From<ArrivalSpeedType> for ArrivalSpeedTypeValue {
    fn from(value: ArrivalSpeedType) -> Self {
        value.value
    }
}
impl Deref for ArrivalSpeedType {
    type Target = ArrivalSpeedTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for ArrivalSpeedType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ArrivalSpeedTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "current")]
    Current,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AssignmentType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@check")]
    pub check: String,
    #[serde(rename = "@value")]
    pub value: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BoolOptionType {
    #[serde(rename = "@value")]
    pub value: BoolType,
    #[serde(default, rename = "@synonymes")]
    pub synonymes: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@help")]
    pub help: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BoolType {
    #[serde(rename = "#text")]
    pub value: BoolTypeValue,
}
impl From<BoolTypeValue> for BoolType {
    fn from(value: BoolTypeValue) -> Self {
        Self { value }
    }
}
impl From<BoolType> for BoolTypeValue {
    fn from(value: BoolType) -> Self {
        value.value
    }
}
impl Deref for BoolType {
    type Target = BoolTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for BoolType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum BoolTypeValue {
    #[serde(rename = "true")]
    SmallTrue,
    #[serde(rename = "false")]
    SmallFalse,
    #[serde(rename = "True")]
    True,
    #[serde(rename = "False")]
    False,
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "on")]
    On,
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "1")]
    _1,
    #[serde(rename = "0")]
    _0,
    #[serde(rename = "x")]
    X,
    #[serde(rename = "-")]
    Minus,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfAccType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@speedControlGain")]
    pub speed_control_gain: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainSpeed")]
    pub gap_closing_control_gain_speed: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainSpace")]
    pub gap_closing_control_gain_space: Option<String>,
    #[serde(default, rename = "@gapControlGainSpeed")]
    pub gap_control_gain_speed: Option<String>,
    #[serde(default, rename = "@gapControlGainSpace")]
    pub gap_control_gain_space: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainSpeed")]
    pub collision_avoidance_gain_speed: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainSpace")]
    pub collision_avoidance_gain_space: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfBKernerType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@sigma")]
    pub sigma: Option<f32>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@k")]
    pub k: Option<String>,
    #[serde(default, rename = "@phi")]
    pub phi: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfCaccType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@emergencyDecel")]
    pub emergency_decel: Option<String>,
    #[serde(default, rename = "@collisionMinGapFactor")]
    pub collision_min_gap_factor: Option<String>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@speedControlGainCACC")]
    pub speed_control_gain_cacc: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainGap")]
    pub gap_closing_control_gain_gap: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainGapDot")]
    pub gap_closing_control_gain_gap_dot: Option<String>,
    #[serde(default, rename = "@gapControlGainGap")]
    pub gap_control_gain_gap: Option<String>,
    #[serde(default, rename = "@gapControlGainGapDot")]
    pub gap_control_gain_gap_dot: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainGap")]
    pub collision_avoidance_gain_gap: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainGapDot")]
    pub collision_avoidance_gain_gap_dot: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainSpeed")]
    pub gap_closing_control_gain_speed: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainSpace")]
    pub gap_closing_control_gain_space: Option<String>,
    #[serde(default, rename = "@gapControlGainSpeed")]
    pub gap_control_gain_speed: Option<String>,
    #[serde(default, rename = "@gapControlGainSpace")]
    pub gap_control_gain_space: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainSpeed")]
    pub collision_avoidance_gain_speed: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainSpace")]
    pub collision_avoidance_gain_space: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfCcType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@c1")]
    pub c1: Option<String>,
    #[serde(default, rename = "@ccDecel")]
    pub cc_decel: Option<String>,
    #[serde(default, rename = "@constSpacing")]
    pub const_spacing: Option<String>,
    #[serde(default, rename = "@kp")]
    pub kp: Option<String>,
    #[serde(default, rename = "@lambda")]
    pub lambda: Option<String>,
    #[serde(default, rename = "@omegaN")]
    pub omega_n: Option<String>,
    #[serde(default, rename = "@tauEngine")]
    pub tau_engine: Option<String>,
    #[serde(default, rename = "@xi")]
    pub xi: Option<String>,
    #[serde(default, rename = "@lanesCount")]
    pub lanes_count: Option<String>,
    #[serde(default, rename = "@ccAccel")]
    pub cc_accel: Option<String>,
    #[serde(default, rename = "@ploegKp")]
    pub ploeg_kp: Option<String>,
    #[serde(default, rename = "@ploegKd")]
    pub ploeg_kd: Option<String>,
    #[serde(default, rename = "@ploegH")]
    pub ploeg_h: Option<String>,
    #[serde(default, rename = "@flatbedKa")]
    pub flatbed_ka: Option<String>,
    #[serde(default, rename = "@flatbedKv")]
    pub flatbed_kv: Option<String>,
    #[serde(default, rename = "@flatbedKp")]
    pub flatbed_kp: Option<String>,
    #[serde(default, rename = "@flatbedD")]
    pub flatbed_d: Option<String>,
    #[serde(default, rename = "@flatbedH")]
    pub flatbed_h: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfEidmType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@maxAccelProfile")]
    pub max_accel_profile: Option<String>,
    #[serde(default, rename = "@desAccelProfile")]
    pub des_accel_profile: Option<String>,
    #[serde(default, rename = "@stepping")]
    pub stepping: Option<f32>,
    #[serde(default, rename = "@delta")]
    pub delta: Option<String>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@tPersDrive")]
    pub t_pers_drive: Option<f32>,
    #[serde(default, rename = "@tpreview")]
    pub tpreview: Option<f32>,
    #[serde(default, rename = "@treaction")]
    pub treaction: Option<f32>,
    #[serde(default, rename = "@tPersEstimate")]
    pub t_pers_estimate: Option<f32>,
    #[serde(default, rename = "@ccoolness")]
    pub ccoolness: Option<f32>,
    #[serde(default, rename = "@sigmaleader")]
    pub sigmaleader: Option<f32>,
    #[serde(default, rename = "@sigmagap")]
    pub sigmagap: Option<f32>,
    #[serde(default, rename = "@sigmaerror")]
    pub sigmaerror: Option<f32>,
    #[serde(default, rename = "@jerkmax")]
    pub jerkmax: Option<f32>,
    #[serde(default, rename = "@epsilonacc")]
    pub epsilonacc: Option<f32>,
    #[serde(default, rename = "@taccmax")]
    pub taccmax: Option<f32>,
    #[serde(default, rename = "@Mflatness")]
    pub mflatness: Option<f32>,
    #[serde(default, rename = "@Mbegin")]
    pub mbegin: Option<f32>,
    #[serde(default, rename = "@vehdynamics")]
    pub vehdynamics: Option<BoolType>,
    #[serde(default, rename = "@maxvehpreview")]
    pub maxvehpreview: Option<i32>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfIdmmType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@adaptTime")]
    pub adapt_time: Option<String>,
    #[serde(default, rename = "@adaptFactor")]
    pub adapt_factor: Option<String>,
    #[serde(default, rename = "@stepping")]
    pub stepping: Option<f32>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfIdmType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@delta")]
    pub delta: Option<String>,
    #[serde(default, rename = "@stepping")]
    pub stepping: Option<f32>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfKraussType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@sigma")]
    pub sigma: Option<f32>,
    #[serde(default, rename = "@sigmaStep")]
    pub sigma_step: Option<f32>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfPWagType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@sigma")]
    pub sigma: Option<f32>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@tauLast")]
    pub tau_last: Option<String>,
    #[serde(default, rename = "@apProb")]
    pub ap_prob: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfSmartType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@sigma")]
    pub sigma: Option<f32>,
    #[serde(default, rename = "@sigmaStep")]
    pub sigma_step: Option<f32>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@tmp1")]
    pub tmp_1: Option<String>,
    #[serde(default, rename = "@tmp2")]
    pub tmp_2: Option<String>,
    #[serde(default, rename = "@tmp3")]
    pub tmp_3: Option<String>,
    #[serde(default, rename = "@tmp4")]
    pub tmp_4: Option<String>,
    #[serde(default, rename = "@tmp5")]
    pub tmp_5: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfW99Type {
    #[serde(default, rename = "@cc1")]
    pub cc_1: Option<String>,
    #[serde(default, rename = "@cc2")]
    pub cc_2: Option<String>,
    #[serde(default, rename = "@cc3")]
    pub cc_3: Option<String>,
    #[serde(default, rename = "@cc4")]
    pub cc_4: Option<String>,
    #[serde(default, rename = "@cc5")]
    pub cc_5: Option<String>,
    #[serde(default, rename = "@cc6")]
    pub cc_6: Option<String>,
    #[serde(default, rename = "@cc7")]
    pub cc_7: Option<String>,
    #[serde(default, rename = "@cc8")]
    pub cc_8: Option<String>,
    #[serde(default, rename = "@cc9")]
    pub cc_9: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CfWiedemannType {
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@security")]
    pub security: Option<String>,
    #[serde(default, rename = "@estimation")]
    pub estimation: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ColorType {
    #[serde(rename = "#text")]
    pub value: ColorTypeValue,
}
impl From<ColorTypeValue> for ColorType {
    fn from(value: ColorTypeValue) -> Self {
        Self { value }
    }
}
impl From<ColorType> for ColorTypeValue {
    fn from(value: ColorType) -> Self {
        value.value
    }
}
impl Deref for ColorType {
    type Target = ColorTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for ColorType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ColorTypeValue {
    #[serde(rename = "String")]
    String(String),
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "yellow")]
    Yellow,
    #[serde(rename = "cyan")]
    Cyan,
    #[serde(rename = "magenta")]
    Magenta,
    #[serde(rename = "orange")]
    Orange,
    #[serde(rename = "white")]
    White,
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "grey")]
    Grey,
    #[serde(rename = "gray")]
    Gray,
    #[serde(rename = "invisible")]
    Invisible,
    #[serde(rename = "random")]
    Random,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ConditionType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@value")]
    pub value: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictType {
    #[serde(rename = "@from")]
    pub from: String,
    #[serde(rename = "@to")]
    pub to: String,
    #[serde(rename = "@fromLane")]
    pub from_lane: u32,
    #[serde(rename = "@toLane")]
    pub to_lane: u32,
    #[serde(rename = "@startPos")]
    pub start_pos: String,
    #[serde(rename = "@endPos")]
    pub end_pos: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerBaseType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@arrival")]
    pub arrival: Option<f32>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(rename = "#content")]
    pub content: Vec<ContainerBaseTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ContainerBaseTypeContent {
    #[serde(rename = "transport")]
    Transport(ContainerBaseTypeTransportElementType),
    #[serde(rename = "tranship")]
    Tranship(ContainerBaseTypeTranshipElementType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerFlowType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@arrival")]
    pub arrival: Option<f32>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@begin")]
    pub begin: Option<PersonDepartType>,
    #[serde(default, rename = "@end")]
    pub end: Option<TimeType>,
    #[serde(default, rename = "@period")]
    pub period: Option<PeriodType>,
    #[serde(default, rename = "@containersPerHour")]
    pub containers_per_hour: Option<f32>,
    #[serde(default, rename = "@perHour")]
    pub per_hour: Option<f32>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<f32>,
    #[serde(default, rename = "@number")]
    pub number: Option<i32>,
    #[serde(rename = "#content")]
    pub content: Vec<ContainerFlowTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ContainerFlowTypeContent {
    #[serde(rename = "transport")]
    Transport(ContainerBaseTypeTransportElementType),
    #[serde(rename = "tranship")]
    Tranship(ContainerBaseTypeTranshipElementType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@arrival")]
    pub arrival: Option<f32>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(rename = "@depart")]
    pub depart: PersonDepartType,
    #[serde(rename = "#content")]
    pub content: Vec<ContainerTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum ContainerTypeContent {
    #[serde(rename = "transport")]
    Transport(ContainerBaseTypeTransportElementType),
    #[serde(rename = "tranship")]
    Tranship(ContainerBaseTypeTranshipElementType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DepartLaneType {
    #[serde(rename = "#text")]
    pub value: DepartLaneTypeValue,
}
impl From<DepartLaneTypeValue> for DepartLaneType {
    fn from(value: DepartLaneTypeValue) -> Self {
        Self { value }
    }
}
impl From<DepartLaneType> for DepartLaneTypeValue {
    fn from(value: DepartLaneType) -> Self {
        value.value
    }
}
impl Deref for DepartLaneType {
    type Target = DepartLaneTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for DepartLaneType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum DepartLaneTypeValue {
    #[serde(rename = "BigUint")]
    BigUint(u32),
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "allowed")]
    Allowed,
    #[serde(rename = "first")]
    First,
    #[serde(rename = "best")]
    Best,
    #[serde(rename = "best_prob")]
    BestProb,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DepartPosLatType {
    #[serde(rename = "#text")]
    pub value: DepartPosLatTypeValue,
}
impl From<DepartPosLatTypeValue> for DepartPosLatType {
    fn from(value: DepartPosLatTypeValue) -> Self {
        Self { value }
    }
}
impl From<DepartPosLatType> for DepartPosLatTypeValue {
    fn from(value: DepartPosLatType) -> Self {
        value.value
    }
}
impl Deref for DepartPosLatType {
    type Target = DepartPosLatTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for DepartPosLatType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum DepartPosLatTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "random_free")]
    RandomFree,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "left")]
    Left,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DepartPosType {
    #[serde(rename = "#text")]
    pub value: DepartPosTypeValue,
}
impl From<DepartPosTypeValue> for DepartPosType {
    fn from(value: DepartPosTypeValue) -> Self {
        Self { value }
    }
}
impl From<DepartPosType> for DepartPosTypeValue {
    fn from(value: DepartPosType) -> Self {
        value.value
    }
}
impl Deref for DepartPosType {
    type Target = DepartPosTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for DepartPosType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum DepartPosTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "random_free")]
    RandomFree,
    #[serde(rename = "random_location")]
    RandomLocation,
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "base")]
    Base,
    #[serde(rename = "last")]
    Last,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "splitFront")]
    SplitFront,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DepartSpeedType {
    #[serde(rename = "#text")]
    pub value: DepartSpeedTypeValue,
}
impl From<DepartSpeedTypeValue> for DepartSpeedType {
    fn from(value: DepartSpeedTypeValue) -> Self {
        Self { value }
    }
}
impl From<DepartSpeedType> for DepartSpeedTypeValue {
    fn from(value: DepartSpeedType) -> Self {
        value.value
    }
}
impl Deref for DepartSpeedType {
    type Target = DepartSpeedTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for DepartSpeedType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum DepartSpeedTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "desired")]
    Desired,
    #[serde(rename = "speedLimit")]
    SpeedLimit,
    #[serde(rename = "last")]
    Last,
    #[serde(rename = "avg")]
    Avg,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DepartType {
    #[serde(rename = "#text")]
    pub value: DepartTypeValue,
}
impl From<DepartTypeValue> for DepartType {
    fn from(value: DepartTypeValue) -> Self {
        Self { value }
    }
}
impl From<DepartType> for DepartTypeValue {
    fn from(value: DepartType) -> Self {
        value.value
    }
}
impl Deref for DepartType {
    type Target = DepartTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for DepartType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum DepartTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "String")]
    String(String),
    #[serde(rename = "triggered")]
    Triggered,
    #[serde(rename = "containerTriggered")]
    ContainerTriggered,
    #[serde(rename = "split")]
    Split,
    #[serde(rename = "begin")]
    Begin,
}
pub type DetectorIdType = String;
#[derive(Debug, Serialize, Deserialize)]
pub struct FileOptionType {
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(default, rename = "@synonymes")]
    pub synonymes: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@help")]
    pub help: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FloatOptionType {
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(default, rename = "@synonymes")]
    pub synonymes: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@help")]
    pub help: Option<String>,
}
pub type FloatType = String;
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowCalibratorType {
    #[serde(default, rename = "@route")]
    pub route: Option<String>,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@via")]
    pub via: Option<String>,
    #[serde(default, rename = "@fromJunction")]
    pub from_junction: Option<String>,
    #[serde(default, rename = "@toJunction")]
    pub to_junction: Option<String>,
    #[serde(default, rename = "@viaJunctions")]
    pub via_junctions: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@begin")]
    pub begin: Option<TimeType>,
    #[serde(default, rename = "@end")]
    pub end: Option<TimeType>,
    #[serde(default, rename = "@period")]
    pub period: Option<PeriodType>,
    #[serde(default, rename = "@vehsPerHour")]
    pub vehs_per_hour: Option<f32>,
    #[serde(default, rename = "@perHour")]
    pub per_hour: Option<f32>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<f32>,
    #[serde(default, rename = "@number")]
    pub number: Option<i32>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@departLane")]
    pub depart_lane: Option<DepartLaneType>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@departSpeed")]
    pub depart_speed: Option<DepartSpeedType>,
    #[serde(default, rename = "@arrivalLane")]
    pub arrival_lane: Option<ArrivalLaneType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@arrivalSpeed")]
    pub arrival_speed: Option<ArrivalSpeedType>,
    #[serde(default, rename = "@departPosLat")]
    pub depart_pos_lat: Option<DepartPosLatType>,
    #[serde(default, rename = "@arrivalPosLat")]
    pub arrival_pos_lat: Option<ArrivalPosLatType>,
    #[serde(default, rename = "@line")]
    pub line: Option<String>,
    #[serde(default, rename = "@personNumber")]
    pub person_number: Option<u32>,
    #[serde(default, rename = "@containerNumber")]
    pub container_number: Option<u32>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<f32>,
    #[serde(default, rename = "@parkingBadges")]
    pub parking_badges: Option<String>,
    #[serde(default, rename = "@insertionChecks")]
    pub insertion_checks: Option<String>,
    #[serde(default, rename = "@speed")]
    pub speed: Option<f32>,
    #[serde(default, rename = "#content")]
    pub content: Vec<FlowCalibratorTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum FlowCalibratorTypeContent {
    #[serde(rename = "route")]
    Route(VehicleRouteType),
    #[serde(rename = "routeDistribution")]
    RouteDistribution(VehicleRouteDistributionType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowIntervalType {
    #[serde(default, rename = "@begin")]
    pub begin: Option<TimeType>,
    #[serde(default, rename = "@end")]
    pub end: Option<TimeType>,
    #[serde(default, rename = "#content")]
    pub content: Vec<FlowIntervalTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum FlowIntervalTypeContent {
    #[serde(rename = "flow")]
    Flow(FlowType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowType {
    #[serde(default, rename = "@route")]
    pub route: Option<String>,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@via")]
    pub via: Option<String>,
    #[serde(default, rename = "@fromJunction")]
    pub from_junction: Option<String>,
    #[serde(default, rename = "@toJunction")]
    pub to_junction: Option<String>,
    #[serde(default, rename = "@viaJunctions")]
    pub via_junctions: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@begin")]
    pub begin: Option<TimeType>,
    #[serde(default, rename = "@end")]
    pub end: Option<TimeType>,
    #[serde(default, rename = "@period")]
    pub period: Option<PeriodType>,
    #[serde(default, rename = "@vehsPerHour")]
    pub vehs_per_hour: Option<f32>,
    #[serde(default, rename = "@perHour")]
    pub per_hour: Option<f32>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<f32>,
    #[serde(default, rename = "@number")]
    pub number: Option<i32>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@departLane")]
    pub depart_lane: Option<DepartLaneType>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@departSpeed")]
    pub depart_speed: Option<DepartSpeedType>,
    #[serde(default, rename = "@arrivalLane")]
    pub arrival_lane: Option<ArrivalLaneType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@arrivalSpeed")]
    pub arrival_speed: Option<ArrivalSpeedType>,
    #[serde(default, rename = "@departPosLat")]
    pub depart_pos_lat: Option<DepartPosLatType>,
    #[serde(default, rename = "@arrivalPosLat")]
    pub arrival_pos_lat: Option<ArrivalPosLatType>,
    #[serde(default, rename = "@line")]
    pub line: Option<String>,
    #[serde(default, rename = "@personNumber")]
    pub person_number: Option<u32>,
    #[serde(default, rename = "@containerNumber")]
    pub container_number: Option<u32>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<f32>,
    #[serde(default, rename = "@parkingBadges")]
    pub parking_badges: Option<String>,
    #[serde(default, rename = "@insertionChecks")]
    pub insertion_checks: Option<String>,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@reroute")]
    pub reroute: Option<BoolType>,
    #[serde(default, rename = "@departEdge")]
    pub depart_edge: Option<RouteIndexType>,
    #[serde(default, rename = "@arrivalEdge")]
    pub arrival_edge: Option<RouteIndexType>,
    #[serde(default, rename = "#content")]
    pub content: Vec<FlowTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum FlowTypeContent {
    #[serde(rename = "route")]
    Route(VehicleRouteType),
    #[serde(rename = "routeDistribution")]
    RouteDistribution(VehicleRouteDistributionType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowWithoutIdType {
    #[serde(default, rename = "@route")]
    pub route: Option<String>,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@via")]
    pub via: Option<String>,
    #[serde(default, rename = "@fromJunction")]
    pub from_junction: Option<String>,
    #[serde(default, rename = "@toJunction")]
    pub to_junction: Option<String>,
    #[serde(default, rename = "@viaJunctions")]
    pub via_junctions: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@begin")]
    pub begin: Option<TimeType>,
    #[serde(default, rename = "@end")]
    pub end: Option<TimeType>,
    #[serde(default, rename = "@period")]
    pub period: Option<PeriodType>,
    #[serde(default, rename = "@vehsPerHour")]
    pub vehs_per_hour: Option<f32>,
    #[serde(default, rename = "@perHour")]
    pub per_hour: Option<f32>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<f32>,
    #[serde(default, rename = "@number")]
    pub number: Option<i32>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@departLane")]
    pub depart_lane: Option<DepartLaneType>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@departSpeed")]
    pub depart_speed: Option<DepartSpeedType>,
    #[serde(default, rename = "@arrivalLane")]
    pub arrival_lane: Option<ArrivalLaneType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@arrivalSpeed")]
    pub arrival_speed: Option<ArrivalSpeedType>,
    #[serde(default, rename = "@departPosLat")]
    pub depart_pos_lat: Option<DepartPosLatType>,
    #[serde(default, rename = "@arrivalPosLat")]
    pub arrival_pos_lat: Option<ArrivalPosLatType>,
    #[serde(default, rename = "@line")]
    pub line: Option<String>,
    #[serde(default, rename = "@personNumber")]
    pub person_number: Option<u32>,
    #[serde(default, rename = "@containerNumber")]
    pub container_number: Option<u32>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<f32>,
    #[serde(default, rename = "@parkingBadges")]
    pub parking_badges: Option<String>,
    #[serde(default, rename = "@insertionChecks")]
    pub insertion_checks: Option<String>,
    #[serde(default, rename = "#content")]
    pub content: Vec<FlowWithoutIdTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum FlowWithoutIdTypeContent {
    #[serde(rename = "route")]
    Route(VehicleRouteType),
    #[serde(rename = "routeDistribution")]
    RouteDistribution(VehicleRouteDistributionType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@nArgs")]
    pub n_args: i32,
    #[serde(rename = "#content")]
    pub content: Vec<FunctionTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum FunctionTypeContent {
    #[serde(rename = "assignment")]
    Assignment(AssignmentType),
}
pub type IdType = String;
#[derive(Debug, Serialize, Deserialize)]
pub struct IncludeType {
    #[serde(default, rename = "@href")]
    pub href: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct IntArrayOptionType {
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(default, rename = "@synonymes")]
    pub synonymes: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@help")]
    pub help: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct IntOptionType {
    #[serde(rename = "@value")]
    pub value: i32,
    #[serde(default, rename = "@synonymes")]
    pub synonymes: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@help")]
    pub help: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LaneTypeType {
    #[serde(rename = "@index")]
    pub index: i32,
    #[serde(default, rename = "@allow")]
    pub allow: Option<String>,
    #[serde(default, rename = "@disallow")]
    pub disallow: Option<String>,
    #[serde(default, rename = "@speed")]
    pub speed: Option<String>,
    #[serde(default, rename = "@width")]
    pub width: Option<String>,
    #[serde(default, rename = "#content")]
    pub content: Vec<LaneTypeTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum LaneTypeTypeContent {
    #[serde(rename = "restriction")]
    Restriction(RestrictionType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LatAlignmentType {
    #[serde(rename = "#text")]
    pub value: LatAlignmentTypeValue,
}
impl From<LatAlignmentTypeValue> for LatAlignmentType {
    fn from(value: LatAlignmentTypeValue) -> Self {
        Self { value }
    }
}
impl From<LatAlignmentType> for LatAlignmentTypeValue {
    fn from(value: LatAlignmentType) -> Self {
        value.value
    }
}
impl Deref for LatAlignmentType {
    type Target = LatAlignmentTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for LatAlignmentType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum LatAlignmentTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "arbitrary")]
    Arbitrary,
    #[serde(rename = "nice")]
    Nice,
    #[serde(rename = "compact")]
    Compact,
    #[serde(rename = "left")]
    Left,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LocationType {
    #[serde(default, rename = "@netOffset")]
    pub net_offset: Option<String>,
    #[serde(default, rename = "@convBoundary")]
    pub conv_boundary: Option<String>,
    #[serde(default, rename = "@origBoundary")]
    pub orig_boundary: Option<String>,
    #[serde(rename = "@projParameter")]
    pub proj_parameter: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MesoType {
    #[serde(default, rename = "@tauff")]
    pub tauff: Option<f32>,
    #[serde(default, rename = "@taufj")]
    pub taufj: Option<f32>,
    #[serde(default, rename = "@taujf")]
    pub taujf: Option<f32>,
    #[serde(default, rename = "@taujj")]
    pub taujj: Option<f32>,
    #[serde(default, rename = "@jamThreshold")]
    pub jam_threshold: Option<String>,
    #[serde(default, rename = "@junctionControl")]
    pub junction_control: Option<BoolType>,
    #[serde(default, rename = "@tlsPenalty")]
    pub tls_penalty: Option<f32>,
    #[serde(default, rename = "@tlsFlowPenalty")]
    pub tls_flow_penalty: Option<f32>,
    #[serde(default, rename = "@minorPenalty")]
    pub minor_penalty: Option<f32>,
    #[serde(default, rename = "@overtaking")]
    pub overtaking: Option<BoolType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTypeType {
    #[serde(rename = "#text")]
    pub value: NodeTypeTypeValue,
}
impl From<NodeTypeTypeValue> for NodeTypeType {
    fn from(value: NodeTypeTypeValue) -> Self {
        Self { value }
    }
}
impl From<NodeTypeType> for NodeTypeTypeValue {
    fn from(value: NodeTypeType) -> Self {
        value.value
    }
}
impl Deref for NodeTypeType {
    type Target = NodeTypeTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for NodeTypeType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum NodeTypeTypeValue {
    #[serde(rename = "traffic_light")]
    TrafficLight,
    #[serde(rename = "right_before_left")]
    RightBeforeLeft,
    #[serde(rename = "left_before_right")]
    LeftBeforeRight,
    #[serde(rename = "priority")]
    Priority,
    #[serde(rename = "dead_end")]
    DeadEnd,
    #[serde(rename = "unregulated")]
    Unregulated,
    #[serde(rename = "traffic_light_unregulated")]
    TrafficLightUnregulated,
    #[serde(rename = "rail_signal")]
    RailSignal,
    #[serde(rename = "allway_stop")]
    AllwayStop,
    #[serde(rename = "priority_stop")]
    PriorityStop,
    #[serde(rename = "zipper")]
    Zipper,
    #[serde(rename = "rail_crossing")]
    RailCrossing,
    #[serde(rename = "traffic_light_right_on_red")]
    TrafficLightRightOnRed,
    #[serde(rename = "district")]
    District,
    #[serde(rename = "unknown")]
    Unknown,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum NonNegativeDistributionType {
    F32(f32),
    String(String),
}
pub type NonNegativeFloatType = f32;
pub type NonNegativeFloatTypeWithErrorValueType = f32;
pub type NonNegativeIntType = i32;
#[derive(Debug, Serialize, Deserialize)]
pub struct OffsetType {
    #[serde(rename = "#text")]
    pub value: OffsetTypeValue,
}
impl From<OffsetTypeValue> for OffsetType {
    fn from(value: OffsetTypeValue) -> Self {
        Self { value }
    }
}
impl From<OffsetType> for OffsetTypeValue {
    fn from(value: OffsetType) -> Self {
        value.value
    }
}
impl Deref for OffsetType {
    type Target = OffsetTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for OffsetType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum OffsetTypeValue {
    #[serde(rename = "String")]
    String(String),
    #[serde(rename = "begin")]
    Begin,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ParamType {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(default, rename = "@value")]
    pub value: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PeriodType {
    F32(f32),
    String(String),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonBaseType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@arrival")]
    pub arrival: Option<TimeType>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@modes")]
    pub modes: Option<String>,
    #[serde(default, rename = "@vTypes")]
    pub v_types: Option<String>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<f32>,
    #[serde(rename = "#content")]
    pub content: Vec<PersonBaseTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PersonBaseTypeContent {
    #[serde(rename = "personTrip")]
    PersonTrip(PersonBaseTypePersonTripElementType),
    #[serde(rename = "ride")]
    Ride(PersonBaseTypeRideElementType),
    #[serde(rename = "walk")]
    Walk(PersonBaseTypeWalkElementType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonDepartType {
    #[serde(rename = "#text")]
    pub value: PersonDepartTypeValue,
}
impl From<PersonDepartTypeValue> for PersonDepartType {
    fn from(value: PersonDepartTypeValue) -> Self {
        Self { value }
    }
}
impl From<PersonDepartType> for PersonDepartTypeValue {
    fn from(value: PersonDepartType) -> Self {
        value.value
    }
}
impl Deref for PersonDepartType {
    type Target = PersonDepartTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for PersonDepartType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PersonDepartTypeValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "String")]
    String(String),
    #[serde(rename = "triggered")]
    Triggered,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonFlowType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@arrival")]
    pub arrival: Option<TimeType>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@modes")]
    pub modes: Option<String>,
    #[serde(default, rename = "@vTypes")]
    pub v_types: Option<String>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<f32>,
    #[serde(default, rename = "@begin")]
    pub begin: Option<PersonDepartType>,
    #[serde(default, rename = "@end")]
    pub end: Option<TimeType>,
    #[serde(default, rename = "@period")]
    pub period: Option<PeriodType>,
    #[serde(default, rename = "@vehsPerHour")]
    pub vehs_per_hour: Option<f32>,
    #[serde(default, rename = "@personsPerHour")]
    pub persons_per_hour: Option<f32>,
    #[serde(default, rename = "@perHour")]
    pub per_hour: Option<f32>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<f32>,
    #[serde(default, rename = "@number")]
    pub number: Option<i32>,
    #[serde(rename = "#content")]
    pub content: Vec<PersonFlowTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PersonFlowTypeContent {
    #[serde(rename = "personTrip")]
    PersonTrip(PersonBaseTypePersonTripElementType),
    #[serde(rename = "ride")]
    Ride(PersonBaseTypeRideElementType),
    #[serde(rename = "walk")]
    Walk(PersonBaseTypeWalkElementType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@arrival")]
    pub arrival: Option<TimeType>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@modes")]
    pub modes: Option<String>,
    #[serde(default, rename = "@vTypes")]
    pub v_types: Option<String>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<f32>,
    #[serde(rename = "@depart")]
    pub depart: PersonDepartType,
    #[serde(rename = "#content")]
    pub content: Vec<PersonTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PersonTypeContent {
    #[serde(rename = "personTrip")]
    PersonTrip(PersonBaseTypePersonTripElementType),
    #[serde(rename = "ride")]
    Ride(PersonBaseTypeRideElementType),
    #[serde(rename = "walk")]
    Walk(PersonBaseTypeWalkElementType),
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PhaseType {
    #[serde(rename = "@duration")]
    pub duration: f32,
    #[serde(default, rename = "@minDur")]
    pub min_dur: Option<f32>,
    #[serde(default, rename = "@maxDur")]
    pub max_dur: Option<f32>,
    #[serde(default, rename = "@earliestEnd")]
    pub earliest_end: Option<f32>,
    #[serde(default, rename = "@latestEnd")]
    pub latest_end: Option<f32>,
    #[serde(default, rename = "@earlyTarget")]
    pub early_target: Option<String>,
    #[serde(default, rename = "@finalTarget")]
    pub final_target: Option<String>,
    #[serde(default, rename = "@yellow")]
    pub yellow: Option<f32>,
    #[serde(default, rename = "@red")]
    pub red: Option<f32>,
    #[serde(default, rename = "@vehext")]
    pub vehext: Option<f32>,
    #[serde(rename = "@state")]
    pub state: String,
    #[serde(default, rename = "@next")]
    pub next: Option<String>,
    #[serde(default, rename = "@name")]
    pub name: Option<String>,
}
pub type PositionType = String;
pub type PositiveFloatType = f32;
pub type PositiveIntType = i32;
#[derive(Debug, Serialize, Deserialize)]
pub struct RestrictionType {
    #[serde(rename = "@vClass")]
    pub v_class: String,
    #[serde(rename = "@speed")]
    pub speed: String,
}
pub type RouteAlternatives = RoutesType;
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteDistRouteType {
    #[serde(default, rename = "@id")]
    pub id: Option<String>,
    #[serde(default, rename = "@edges")]
    pub edges: Option<String>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@cost")]
    pub cost: Option<String>,
    #[serde(default, rename = "@savings")]
    pub savings: Option<String>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<String>,
    #[serde(default, rename = "@exitTimes")]
    pub exit_times: Option<String>,
    #[serde(default, rename = "@refId")]
    pub ref_id: Option<String>,
    #[serde(default, rename = "@reason")]
    pub reason: Option<String>,
    #[serde(default, rename = "@replacedOnEdge")]
    pub replaced_on_edge: Option<String>,
    #[serde(default, rename = "@replacedOnIndex")]
    pub replaced_on_index: Option<f32>,
    #[serde(default, rename = "@replacedAtTime")]
    pub replaced_at_time: Option<TimeType>,
    #[serde(default, rename = "@routeLength")]
    pub route_length: Option<f32>,
    #[serde(default, rename = "stop")]
    pub stop: Vec<StopType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteDistributionType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@last")]
    pub last: Option<u32>,
    #[serde(default, rename = "@routes")]
    pub routes: Option<String>,
    #[serde(default, rename = "@probabilities")]
    pub probabilities: Option<String>,
    #[serde(default, rename = "#content")]
    pub content: Option<RouteDistributionTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteDistributionTypeContent {
    #[serde(default, rename = "route")]
    pub route: Vec<RouteDistRouteType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteIndexType {
    #[serde(rename = "#text")]
    pub value: RouteIndexTypeValue,
}
impl From<RouteIndexTypeValue> for RouteIndexType {
    fn from(value: RouteIndexTypeValue) -> Self {
        Self { value }
    }
}
impl From<RouteIndexType> for RouteIndexTypeValue {
    fn from(value: RouteIndexType) -> Self {
        value.value
    }
}
impl Deref for RouteIndexType {
    type Target = RouteIndexTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for RouteIndexType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum RouteIndexTypeValue {
    #[serde(rename = "BigUint")]
    BigUint(u32),
    #[serde(rename = "random")]
    Random,
    #[serde(rename = "free")]
    Free,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RouteType {
    #[serde(default, rename = "@edges")]
    pub edges: Option<String>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@exitTimes")]
    pub exit_times: Option<String>,
    #[serde(default, rename = "@cost")]
    pub cost: Option<String>,
    #[serde(default, rename = "@savings")]
    pub savings: Option<String>,
    #[serde(default, rename = "@repeat")]
    pub repeat: Option<u32>,
    #[serde(default, rename = "@cycleTime")]
    pub cycle_time: Option<TimeType>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<String>,
    #[serde(default, rename = "@routeLength")]
    pub route_length: Option<f32>,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "#content")]
    pub content: Vec<RouteTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum RouteTypeContent {
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
pub type Routes = RoutesType;
#[derive(Debug, Serialize, Deserialize)]
pub struct RoutesType {
    #[serde(default, rename = "#content")]
    pub content: Vec<RoutesTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum RoutesTypeContent {
    #[serde(rename = "vTypeDistribution")]
    VTypeDistribution(VTypeDistributionType),
    #[serde(rename = "routeDistribution")]
    RouteDistribution(RouteDistributionType),
    #[serde(rename = "vType")]
    VType(VTypeType),
    #[serde(rename = "vehicle")]
    Vehicle(VehicleType),
    #[serde(rename = "route")]
    Route(RouteType),
    #[serde(rename = "flow")]
    Flow(FlowType),
    #[serde(rename = "trip")]
    Trip(TripType),
    #[serde(rename = "person")]
    Person(PersonType),
    #[serde(rename = "personFlow")]
    PersonFlow(PersonFlowType),
    #[serde(rename = "container")]
    Container(ContainerType),
    #[serde(rename = "containerFlow")]
    ContainerFlow(ContainerFlowType),
    #[serde(rename = "interval")]
    Interval(FlowIntervalType),
    #[serde(rename = "include")]
    Include(IncludeType),
}
pub type ShapeType = String;
pub type ShapeTypeTwoType = String;
pub type SignedTimeType = String;
#[derive(Debug, Serialize, Deserialize)]
pub struct SplitType {
    #[serde(default, rename = "@lanes")]
    pub lanes: Option<String>,
    #[serde(rename = "@pos")]
    pub pos: String,
    #[serde(default, rename = "@speed")]
    pub speed: Option<f32>,
    #[serde(default, rename = "@type")]
    pub type_: Option<NodeTypeType>,
    #[serde(default, rename = "@tl")]
    pub tl: Option<String>,
    #[serde(default, rename = "@tlType")]
    pub tl_type: Option<TlTypeType>,
    #[serde(default, rename = "@shape")]
    pub shape: Option<String>,
    #[serde(default, rename = "@radius")]
    pub radius: Option<f32>,
    #[serde(default, rename = "@keepClear")]
    pub keep_clear: Option<BoolType>,
    #[serde(default, rename = "@id")]
    pub id: Option<String>,
    #[serde(default, rename = "@idBefore")]
    pub id_before: Option<String>,
    #[serde(default, rename = "@idAfter")]
    pub id_after: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StopType {
    #[serde(default, rename = "@lane")]
    pub lane: Option<String>,
    #[serde(default, rename = "@edge")]
    pub edge: Option<String>,
    #[serde(default, rename = "@lon")]
    pub lon: Option<String>,
    #[serde(default, rename = "@lat")]
    pub lat: Option<String>,
    #[serde(default, rename = "@busStop")]
    pub bus_stop: Option<String>,
    #[serde(default, rename = "@trainStop")]
    pub train_stop: Option<String>,
    #[serde(default, rename = "@containerStop")]
    pub container_stop: Option<String>,
    #[serde(default, rename = "@chargingStation")]
    pub charging_station: Option<String>,
    #[serde(default, rename = "@parkingArea")]
    pub parking_area: Option<String>,
    #[serde(default, rename = "@startPos")]
    pub start_pos: Option<String>,
    #[serde(default, rename = "@endPos")]
    pub end_pos: Option<String>,
    #[serde(default, rename = "@posLat")]
    pub pos_lat: Option<String>,
    #[serde(default, rename = "@friendlyPos")]
    pub friendly_pos: Option<BoolType>,
    #[serde(default, rename = "@arrival")]
    pub arrival: Option<TimeType>,
    #[serde(default, rename = "@duration")]
    pub duration: Option<TimeType>,
    #[serde(default, rename = "@until")]
    pub until: Option<TimeType>,
    #[serde(default, rename = "@extension")]
    pub extension: Option<TimeType>,
    #[serde(default, rename = "@index")]
    pub index: Option<String>,
    #[serde(default, rename = "@parking")]
    pub parking: Option<StopTypeParkingType>,
    #[serde(default, rename = "@triggered")]
    pub triggered: Option<String>,
    #[serde(default, rename = "@containerTriggered")]
    pub container_triggered: Option<BoolType>,
    #[serde(default, rename = "@expected")]
    pub expected: Option<String>,
    #[serde(default, rename = "@permitted")]
    pub permitted: Option<String>,
    #[serde(default, rename = "@expectedContainers")]
    pub expected_containers: Option<String>,
    #[serde(default, rename = "@actType")]
    pub act_type: Option<String>,
    #[serde(default, rename = "@tripId")]
    pub trip_id: Option<String>,
    #[serde(default, rename = "@split")]
    pub split: Option<String>,
    #[serde(default, rename = "@join")]
    pub join: Option<String>,
    #[serde(default, rename = "@line")]
    pub line: Option<String>,
    #[serde(default, rename = "@speed")]
    pub speed: Option<f32>,
    #[serde(default, rename = "@priorEdges")]
    pub prior_edges: Option<String>,
    #[serde(default, rename = "@priorEdgesLength")]
    pub prior_edges_length: Option<String>,
    #[serde(default, rename = "@started")]
    pub started: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "@ended")]
    pub ended: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "@onDemand")]
    pub on_demand: Option<BoolType>,
    #[serde(default, rename = "@jump")]
    pub jump: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "@jumpUntil")]
    pub jump_until: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "@priority")]
    pub priority: Option<f32>,
    #[serde(default, rename = "@actualArrival")]
    pub actual_arrival: Option<TimeType>,
    #[serde(default, rename = "@depart")]
    pub depart: Option<TimeType>,
    #[serde(default, rename = "@collision")]
    pub collision: Option<BoolType>,
    #[serde(default, rename = "param")]
    pub param: Vec<ParamType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StrArrayOptionType {
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(default, rename = "@synonymes")]
    pub synonymes: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@help")]
    pub help: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StrOptionType {
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(default, rename = "@synonymes")]
    pub synonymes: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@help")]
    pub help: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeOptionType {
    #[serde(rename = "@value")]
    pub value: TimeTypeWithErrorValueType,
    #[serde(default, rename = "@synonymes")]
    pub synonymes: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(default, rename = "@help")]
    pub help: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TimeType {
    F32(f32),
    String(String),
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TimeTypeWithErrorValueType {
    F32(f32),
    String(String),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TlLogicType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@type")]
    pub type_: Option<TlTypeType>,
    #[serde(rename = "@programID")]
    pub program_id: String,
    #[serde(default, rename = "@offset")]
    pub offset: Option<OffsetType>,
    #[serde(default, rename = "#content")]
    pub content: Vec<TlLogicTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TlLogicTypeContent {
    #[serde(rename = "phase")]
    Phase(PhaseType),
    #[serde(rename = "param")]
    Param(ParamType),
    #[serde(rename = "condition")]
    Condition(ConditionType),
    #[serde(rename = "assignment")]
    Assignment(AssignmentType),
    #[serde(rename = "function")]
    Function(FunctionType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TlTypeType {
    #[serde(rename = "#text")]
    pub value: TlTypeTypeValue,
}
impl From<TlTypeTypeValue> for TlTypeType {
    fn from(value: TlTypeTypeValue) -> Self {
        Self { value }
    }
}
impl From<TlTypeType> for TlTypeTypeValue {
    fn from(value: TlTypeType) -> Self {
        value.value
    }
}
impl Deref for TlTypeType {
    type Target = TlTypeTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for TlTypeType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TlTypeTypeValue {
    #[serde(rename = "actuated")]
    Actuated,
    #[serde(rename = "delay_based")]
    DelayBased,
    #[serde(rename = "static")]
    Static,
    #[serde(rename = "NEMA")]
    Nema,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TripType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@via")]
    pub via: Option<String>,
    #[serde(default, rename = "@fromLonLat")]
    pub from_lon_lat: Option<String>,
    #[serde(default, rename = "@toLonLat")]
    pub to_lon_lat: Option<String>,
    #[serde(default, rename = "@viaLonLat")]
    pub via_lon_lat: Option<String>,
    #[serde(default, rename = "@fromXY")]
    pub from_xy: Option<String>,
    #[serde(default, rename = "@toXY")]
    pub to_xy: Option<String>,
    #[serde(default, rename = "@viaXY")]
    pub via_xy: Option<String>,
    #[serde(default, rename = "@fromJunction")]
    pub from_junction: Option<String>,
    #[serde(default, rename = "@toJunction")]
    pub to_junction: Option<String>,
    #[serde(default, rename = "@viaJunctions")]
    pub via_junctions: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(rename = "@depart")]
    pub depart: DepartType,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@departLane")]
    pub depart_lane: Option<DepartLaneType>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@departSpeed")]
    pub depart_speed: Option<DepartSpeedType>,
    #[serde(default, rename = "@departEdge")]
    pub depart_edge: Option<RouteIndexType>,
    #[serde(default, rename = "@arrivalEdge")]
    pub arrival_edge: Option<RouteIndexType>,
    #[serde(default, rename = "@arrivalLane")]
    pub arrival_lane: Option<ArrivalLaneType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@arrivalSpeed")]
    pub arrival_speed: Option<ArrivalSpeedType>,
    #[serde(default, rename = "@departPosLat")]
    pub depart_pos_lat: Option<DepartPosLatType>,
    #[serde(default, rename = "@arrivalPosLat")]
    pub arrival_pos_lat: Option<ArrivalPosLatType>,
    #[serde(default, rename = "@line")]
    pub line: Option<String>,
    #[serde(default, rename = "@personNumber")]
    pub person_number: Option<u32>,
    #[serde(default, rename = "@containerNumber")]
    pub container_number: Option<u32>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<f32>,
    #[serde(default, rename = "@parkingBadges")]
    pub parking_badges: Option<String>,
    #[serde(default, rename = "@insertionChecks")]
    pub insertion_checks: Option<String>,
    #[serde(default, rename = "#content")]
    pub content: Vec<TripTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TripTypeContent {
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TypeType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@allow")]
    pub allow: Option<String>,
    #[serde(default, rename = "@disallow")]
    pub disallow: Option<String>,
    #[serde(default, rename = "@priority")]
    pub priority: Option<i32>,
    #[serde(default, rename = "@numLanes")]
    pub num_lanes: Option<i32>,
    #[serde(default, rename = "@speed")]
    pub speed: Option<String>,
    #[serde(default, rename = "@discard")]
    pub discard: Option<BoolType>,
    #[serde(default, rename = "@oneway")]
    pub oneway: Option<BoolType>,
    #[serde(default, rename = "@width")]
    pub width: Option<String>,
    #[serde(default, rename = "@widthResolution")]
    pub width_resolution: Option<f32>,
    #[serde(default, rename = "@maxWidth")]
    pub max_width: Option<f32>,
    #[serde(default, rename = "@minWidth")]
    pub min_width: Option<f32>,
    #[serde(default, rename = "@sidewalkWidth")]
    pub sidewalk_width: Option<String>,
    #[serde(default, rename = "@bikeLaneWidth")]
    pub bike_lane_width: Option<String>,
    #[serde(default, rename = "@spreadType")]
    pub spread_type: Option<TypeTypeSpreadType>,
    #[serde(default, rename = "#content")]
    pub content: Vec<TypeTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TypeTypeContent {
    #[serde(rename = "restriction")]
    Restriction(RestrictionType),
    #[serde(rename = "meso")]
    Meso(MesoType),
    #[serde(rename = "laneType")]
    LaneType(LaneTypeType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VTypeDistributionType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@vTypes")]
    pub v_types: Option<String>,
    #[serde(default, rename = "@probabilities")]
    pub probabilities: Option<String>,
    #[serde(default, rename = "@deterministic")]
    pub deterministic: Option<i32>,
    #[serde(default, rename = "#content")]
    pub content: Option<VTypeDistributionTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VTypeDistributionTypeContent {
    #[serde(default, rename = "vType")]
    pub v_type: Vec<VTypeType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VTypeType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@length")]
    pub length: Option<f32>,
    #[serde(default, rename = "@minGap")]
    pub min_gap: Option<f32>,
    #[serde(default, rename = "@maxSpeed")]
    pub max_speed: Option<f32>,
    #[serde(default, rename = "@desiredMaxSpeed")]
    pub desired_max_speed: Option<f32>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<f32>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<NonNegativeDistributionType>,
    #[serde(default, rename = "@speedDev")]
    pub speed_dev: Option<f32>,
    #[serde(default, rename = "@vClass")]
    pub v_class: Option<String>,
    #[serde(default, rename = "@emissionClass")]
    pub emission_class: Option<String>,
    #[serde(default, rename = "@maneuverAngleTimes")]
    pub maneuver_angle_times: Option<String>,
    #[serde(default, rename = "@guiShape")]
    pub gui_shape: Option<String>,
    #[serde(default, rename = "@width")]
    pub width: Option<f32>,
    #[serde(default, rename = "@height")]
    pub height: Option<f32>,
    #[serde(default, rename = "@mass")]
    pub mass: Option<f32>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@accel")]
    pub accel: Option<f32>,
    #[serde(default, rename = "@decel")]
    pub decel: Option<f32>,
    #[serde(default, rename = "@emergencyDecel")]
    pub emergency_decel: Option<f32>,
    #[serde(default, rename = "@apparentDecel")]
    pub apparent_decel: Option<f32>,
    #[serde(default, rename = "@maxAccelProfile")]
    pub max_accel_profile: Option<String>,
    #[serde(default, rename = "@desAccelProfile")]
    pub des_accel_profile: Option<String>,
    #[serde(default, rename = "@parkingBadges")]
    pub parking_badges: Option<String>,
    #[serde(default, rename = "@personCapacity")]
    pub person_capacity: Option<u32>,
    #[serde(default, rename = "@containerCapacity")]
    pub container_capacity: Option<u32>,
    #[serde(default, rename = "@boardingDuration")]
    pub boarding_duration: Option<f32>,
    #[serde(default, rename = "@loadingDuration")]
    pub loading_duration: Option<f32>,
    #[serde(default, rename = "@scale")]
    pub scale: Option<f32>,
    #[serde(default, rename = "@lcStrategic")]
    pub lc_strategic: Option<String>,
    #[serde(default, rename = "@lcCooperative")]
    pub lc_cooperative: Option<String>,
    #[serde(default, rename = "@lcSpeedGain")]
    pub lc_speed_gain: Option<String>,
    #[serde(default, rename = "@lcKeepRight")]
    pub lc_keep_right: Option<String>,
    #[serde(default, rename = "@lcSublane")]
    pub lc_sublane: Option<String>,
    #[serde(default, rename = "@lcOpposite")]
    pub lc_opposite: Option<String>,
    #[serde(default, rename = "@lcPushy")]
    pub lc_pushy: Option<String>,
    #[serde(default, rename = "@lcPushyGap")]
    pub lc_pushy_gap: Option<f32>,
    #[serde(default, rename = "@lcStrategicLookahead")]
    pub lc_strategic_lookahead: Option<f32>,
    #[serde(default, rename = "@lcAssertive")]
    pub lc_assertive: Option<f32>,
    #[serde(default, rename = "@lcLookaheadLeft")]
    pub lc_lookahead_left: Option<f32>,
    #[serde(default, rename = "@lcSpeedGainRight")]
    pub lc_speed_gain_right: Option<f32>,
    #[serde(default, rename = "@lcSpeedGainLookahead")]
    pub lc_speed_gain_lookahead: Option<f32>,
    #[serde(default, rename = "@lcSpeedGainRemainTime")]
    pub lc_speed_gain_remain_time: Option<f32>,
    #[serde(default, rename = "@lcSpeedGainUrgency")]
    pub lc_speed_gain_urgency: Option<f32>,
    #[serde(default, rename = "@lcCooperativeRoundabout")]
    pub lc_cooperative_roundabout: Option<f32>,
    #[serde(default, rename = "@lcCooperativeSpeed")]
    pub lc_cooperative_speed: Option<f32>,
    #[serde(default, rename = "@lcTurnAlignmentDistance")]
    pub lc_turn_alignment_distance: Option<f32>,
    #[serde(default, rename = "@lcImpatience")]
    pub lc_impatience: Option<f32>,
    #[serde(default, rename = "@lcTimeToImpatience")]
    pub lc_time_to_impatience: Option<f32>,
    #[serde(default, rename = "@lcAccelLat")]
    pub lc_accel_lat: Option<f32>,
    #[serde(default, rename = "@lcMaxSpeedLatStanding")]
    pub lc_max_speed_lat_standing: Option<f32>,
    #[serde(default, rename = "@lcMaxSpeedLatFactor")]
    pub lc_max_speed_lat_factor: Option<String>,
    #[serde(default, rename = "@lcMaxDistLatStanding")]
    pub lc_max_dist_lat_standing: Option<f32>,
    #[serde(default, rename = "@lcOvertakeRight")]
    pub lc_overtake_right: Option<f32>,
    #[serde(default, rename = "@lcLaneDiscipline")]
    pub lc_lane_discipline: Option<f32>,
    #[serde(default, rename = "@lcSigma")]
    pub lc_sigma: Option<f32>,
    #[serde(default, rename = "@lcKeepRightAcceptanceTime")]
    pub lc_keep_right_acceptance_time: Option<String>,
    #[serde(default, rename = "@lcOvertakeDeltaSpeedFactor")]
    pub lc_overtake_delta_speed_factor: Option<String>,
    #[serde(default, rename = "@lcContRight")]
    pub lc_cont_right: Option<String>,
    #[serde(default, rename = "@maxSpeedLat")]
    pub max_speed_lat: Option<f32>,
    #[serde(default, rename = "@latAlignment")]
    pub lat_alignment: Option<LatAlignmentType>,
    #[serde(default, rename = "@actionStepLength")]
    pub action_step_length: Option<f32>,
    #[serde(default, rename = "@hasDriverState")]
    pub has_driver_state: Option<BoolType>,
    #[serde(default, rename = "@minGapLat")]
    pub min_gap_lat: Option<f32>,
    #[serde(default, rename = "@jmCrossingGap")]
    pub jm_crossing_gap: Option<f32>,
    #[serde(default, rename = "@jmDriveAfterYellowTime")]
    pub jm_drive_after_yellow_time: Option<f32>,
    #[serde(default, rename = "@jmDriveAfterRedTime")]
    pub jm_drive_after_red_time: Option<f32>,
    #[serde(default, rename = "@jmDriveRedSpeed")]
    pub jm_drive_red_speed: Option<f32>,
    #[serde(default, rename = "@jmIgnoreKeepClearTime")]
    pub jm_ignore_keep_clear_time: Option<f32>,
    #[serde(default, rename = "@jmIgnoreFoeSpeed")]
    pub jm_ignore_foe_speed: Option<f32>,
    #[serde(default, rename = "@jmIgnoreFoeProb")]
    pub jm_ignore_foe_prob: Option<f32>,
    #[serde(default, rename = "@jmIgnoreJunctionFoeProb")]
    pub jm_ignore_junction_foe_prob: Option<f32>,
    #[serde(default, rename = "@jmSigmaMinor")]
    pub jm_sigma_minor: Option<f32>,
    #[serde(default, rename = "@jmStoplineGap")]
    pub jm_stopline_gap: Option<f32>,
    #[serde(default, rename = "@jmStoplineGapMinor")]
    pub jm_stopline_gap_minor: Option<f32>,
    #[serde(default, rename = "@jmTimegapMinor")]
    pub jm_timegap_minor: Option<f32>,
    #[serde(default, rename = "@jmExtraGap")]
    pub jm_extra_gap: Option<f32>,
    #[serde(default, rename = "@jmAdvance")]
    pub jm_advance: Option<f32>,
    #[serde(default, rename = "@jmStopSignWait")]
    pub jm_stop_sign_wait: Option<f32>,
    #[serde(default, rename = "@jmAllwayStopWait")]
    pub jm_allway_stop_wait: Option<f32>,
    #[serde(default, rename = "@sigma")]
    pub sigma: Option<f32>,
    #[serde(default, rename = "@sigmaStep")]
    pub sigma_step: Option<f32>,
    #[serde(default, rename = "@impatience")]
    pub impatience: Option<VTypeTypeImpatienceType>,
    #[serde(default, rename = "@tau")]
    pub tau: Option<f32>,
    #[serde(default, rename = "@delta")]
    pub delta: Option<String>,
    #[serde(default, rename = "@stepping")]
    pub stepping: Option<f32>,
    #[serde(default, rename = "@adaptTime")]
    pub adapt_time: Option<String>,
    #[serde(default, rename = "@adaptFactor")]
    pub adapt_factor: Option<String>,
    #[serde(default, rename = "@tmp1")]
    pub tmp_1: Option<String>,
    #[serde(default, rename = "@tmp2")]
    pub tmp_2: Option<String>,
    #[serde(default, rename = "@tmp3")]
    pub tmp_3: Option<String>,
    #[serde(default, rename = "@tmp4")]
    pub tmp_4: Option<String>,
    #[serde(default, rename = "@tmp5")]
    pub tmp_5: Option<String>,
    #[serde(default, rename = "@tauLast")]
    pub tau_last: Option<String>,
    #[serde(default, rename = "@apProb")]
    pub ap_prob: Option<String>,
    #[serde(default, rename = "@k")]
    pub k: Option<String>,
    #[serde(default, rename = "@phi")]
    pub phi: Option<String>,
    #[serde(default, rename = "@security")]
    pub security: Option<String>,
    #[serde(default, rename = "@estimation")]
    pub estimation: Option<String>,
    #[serde(default, rename = "@speedControlGain")]
    pub speed_control_gain: Option<String>,
    #[serde(default, rename = "@speedControlMinGap")]
    pub speed_control_min_gap: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainSpeed")]
    pub gap_closing_control_gain_speed: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainSpace")]
    pub gap_closing_control_gain_space: Option<String>,
    #[serde(default, rename = "@gapControlGainSpeed")]
    pub gap_control_gain_speed: Option<String>,
    #[serde(default, rename = "@gapControlGainSpace")]
    pub gap_control_gain_space: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainSpeed")]
    pub collision_avoidance_gain_speed: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainSpace")]
    pub collision_avoidance_gain_space: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceOverride")]
    pub collision_avoidance_override: Option<String>,
    #[serde(default, rename = "@tauCACCToACC")]
    pub tau_cacc_to_acc: Option<String>,
    #[serde(default, rename = "@applyDriverState")]
    pub apply_driver_state: Option<String>,
    #[serde(default, rename = "@carFollowModel")]
    pub car_follow_model: Option<String>,
    #[serde(default, rename = "@trainType")]
    pub train_type: Option<VTypeTypeTrainType>,
    #[serde(default, rename = "@laneChangeModel")]
    pub lane_change_model: Option<VTypeTypeLaneChangeModelType>,
    #[serde(default, rename = "@imgFile")]
    pub img_file: Option<String>,
    #[serde(default, rename = "@osgFile")]
    pub osg_file: Option<String>,
    #[serde(default, rename = "@cc1")]
    pub cc_1: Option<String>,
    #[serde(default, rename = "@cc2")]
    pub cc_2: Option<String>,
    #[serde(default, rename = "@cc3")]
    pub cc_3: Option<String>,
    #[serde(default, rename = "@cc4")]
    pub cc_4: Option<String>,
    #[serde(default, rename = "@cc5")]
    pub cc_5: Option<String>,
    #[serde(default, rename = "@cc6")]
    pub cc_6: Option<String>,
    #[serde(default, rename = "@cc7")]
    pub cc_7: Option<String>,
    #[serde(default, rename = "@cc8")]
    pub cc_8: Option<String>,
    #[serde(default, rename = "@cc9")]
    pub cc_9: Option<String>,
    #[serde(default, rename = "@c1")]
    pub c1: Option<String>,
    #[serde(default, rename = "@ccDecel")]
    pub cc_decel: Option<String>,
    #[serde(default, rename = "@constSpacing")]
    pub const_spacing: Option<String>,
    #[serde(default, rename = "@kp")]
    pub kp: Option<String>,
    #[serde(default, rename = "@lambda")]
    pub lambda: Option<String>,
    #[serde(default, rename = "@omegaN")]
    pub omega_n: Option<String>,
    #[serde(default, rename = "@tauEngine")]
    pub tau_engine: Option<String>,
    #[serde(default, rename = "@xi")]
    pub xi: Option<String>,
    #[serde(default, rename = "@lanesCount")]
    pub lanes_count: Option<String>,
    #[serde(default, rename = "@ccAccel")]
    pub cc_accel: Option<String>,
    #[serde(default, rename = "@ploegKp")]
    pub ploeg_kp: Option<String>,
    #[serde(default, rename = "@ploegKd")]
    pub ploeg_kd: Option<String>,
    #[serde(default, rename = "@ploegH")]
    pub ploeg_h: Option<String>,
    #[serde(default, rename = "@flatbedKa")]
    pub flatbed_ka: Option<String>,
    #[serde(default, rename = "@flatbedKv")]
    pub flatbed_kv: Option<String>,
    #[serde(default, rename = "@flatbedKp")]
    pub flatbed_kp: Option<String>,
    #[serde(default, rename = "@flatbedD")]
    pub flatbed_d: Option<String>,
    #[serde(default, rename = "@flatbedH")]
    pub flatbed_h: Option<String>,
    #[serde(default, rename = "@collisionMinGapFactor")]
    pub collision_min_gap_factor: Option<String>,
    #[serde(default, rename = "@speedControlGainCACC")]
    pub speed_control_gain_cacc: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainGap")]
    pub gap_closing_control_gain_gap: Option<String>,
    #[serde(default, rename = "@gapClosingControlGainGapDot")]
    pub gap_closing_control_gain_gap_dot: Option<String>,
    #[serde(default, rename = "@gapControlGainGap")]
    pub gap_control_gain_gap: Option<String>,
    #[serde(default, rename = "@gapControlGainGapDot")]
    pub gap_control_gain_gap_dot: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainGap")]
    pub collision_avoidance_gain_gap: Option<String>,
    #[serde(default, rename = "@collisionAvoidanceGainGapDot")]
    pub collision_avoidance_gain_gap_dot: Option<String>,
    #[serde(default, rename = "@tPersDrive")]
    pub t_pers_drive: Option<f32>,
    #[serde(default, rename = "@tpreview")]
    pub tpreview: Option<f32>,
    #[serde(default, rename = "@treaction")]
    pub treaction: Option<f32>,
    #[serde(default, rename = "@tPersEstimate")]
    pub t_pers_estimate: Option<f32>,
    #[serde(default, rename = "@ccoolness")]
    pub ccoolness: Option<f32>,
    #[serde(default, rename = "@sigmaleader")]
    pub sigmaleader: Option<f32>,
    #[serde(default, rename = "@sigmagap")]
    pub sigmagap: Option<f32>,
    #[serde(default, rename = "@sigmaerror")]
    pub sigmaerror: Option<f32>,
    #[serde(default, rename = "@jerkmax")]
    pub jerkmax: Option<f32>,
    #[serde(default, rename = "@epsilonacc")]
    pub epsilonacc: Option<f32>,
    #[serde(default, rename = "@taccmax")]
    pub taccmax: Option<f32>,
    #[serde(default, rename = "@Mflatness")]
    pub mflatness: Option<f32>,
    #[serde(default, rename = "@Mbegin")]
    pub mbegin: Option<f32>,
    #[serde(default, rename = "@vehdynamics")]
    pub vehdynamics: Option<BoolType>,
    #[serde(default, rename = "@maxvehpreview")]
    pub maxvehpreview: Option<i32>,
    #[serde(default, rename = "@startupDelay")]
    pub startup_delay: Option<f32>,
    #[serde(default, rename = "@timeToTeleport")]
    pub time_to_teleport: Option<String>,
    #[serde(default, rename = "@timeToTeleportBidi")]
    pub time_to_teleport_bidi: Option<String>,
    #[serde(default, rename = "@speedFactorPremature")]
    pub speed_factor_premature: Option<f32>,
    #[serde(default, rename = "@boardingFactor")]
    pub boarding_factor: Option<String>,
    #[serde(default, rename = "@speedTable")]
    pub speed_table: Option<String>,
    #[serde(default, rename = "@tractionTable")]
    pub traction_table: Option<String>,
    #[serde(default, rename = "@resistanceTable")]
    pub resistance_table: Option<String>,
    #[serde(default, rename = "@massFactor")]
    pub mass_factor: Option<f32>,
    #[serde(default, rename = "@maxPower")]
    pub max_power: Option<f32>,
    #[serde(default, rename = "@maxTraction")]
    pub max_traction: Option<f32>,
    #[serde(default, rename = "@resCoef_constant")]
    pub res_coef_constant: Option<String>,
    #[serde(default, rename = "@resCoef_linear")]
    pub res_coef_linear: Option<String>,
    #[serde(default, rename = "@resCoef_quadratic")]
    pub res_coef_quadratic: Option<String>,
    #[serde(rename = "#content")]
    pub content: Vec<VTypeTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum VTypeTypeContent {
    #[serde(rename = "param")]
    Param(ParamType),
    #[serde(rename = "carFollowing-IDM")]
    CarFollowingIdm(CfIdmType),
    #[serde(rename = "carFollowing-IDMM")]
    CarFollowingIdmm(CfIdmmType),
    #[serde(rename = "carFollowing-EIDM")]
    CarFollowingEidm(CfEidmType),
    #[serde(rename = "carFollowing-Krauss")]
    CarFollowingKrauss(CfKraussType),
    #[serde(rename = "carFollowing-KraussPS")]
    CarFollowingKraussPs(CfKraussType),
    #[serde(rename = "carFollowing-KraussOrig1")]
    CarFollowingKraussOrig1(CfKraussType),
    #[serde(rename = "carFollowing-SmartSK")]
    CarFollowingSmartSk(CfSmartType),
    #[serde(rename = "carFollowing-Daniel1")]
    CarFollowingDaniel1(CfSmartType),
    #[serde(rename = "carFollowing-PWagner2009")]
    CarFollowingPWagner2009(CfPWagType),
    #[serde(rename = "carFollowing-BKerner")]
    CarFollowingBKerner(CfBKernerType),
    #[serde(rename = "carFollowing-Wiedemann")]
    CarFollowingWiedemann(CfWiedemannType),
    #[serde(rename = "carFollowing-W99")]
    CarFollowingW99(CfW99Type),
    #[serde(rename = "carFollowing-ACC")]
    CarFollowingAcc(CfAccType),
    #[serde(rename = "carFollowing-CACC")]
    CarFollowingCacc(CfCaccType),
    #[serde(rename = "carFollowing-CC")]
    CarFollowingCc(CfCcType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VehicleRouteDistributionType {
    #[serde(default, rename = "@id")]
    pub id: Option<String>,
    #[serde(default, rename = "@last")]
    pub last: Option<u32>,
    #[serde(default, rename = "route")]
    pub route: Vec<RouteDistRouteType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VehicleRouteType {
    #[serde(default, rename = "@edges")]
    pub edges: Option<String>,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@exitTimes")]
    pub exit_times: Option<String>,
    #[serde(default, rename = "@cost")]
    pub cost: Option<String>,
    #[serde(default, rename = "@savings")]
    pub savings: Option<String>,
    #[serde(default, rename = "@repeat")]
    pub repeat: Option<u32>,
    #[serde(default, rename = "@cycleTime")]
    pub cycle_time: Option<TimeType>,
    #[serde(default, rename = "@probability")]
    pub probability: Option<String>,
    #[serde(default, rename = "@routeLength")]
    pub route_length: Option<f32>,
    #[serde(default, rename = "#content")]
    pub content: Vec<VehicleRouteTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum VehicleRouteTypeContent {
    #[serde(rename = "stop")]
    Stop(StopType),
    #[serde(rename = "param")]
    Param(ParamType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VehicleType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@route")]
    pub route: Option<String>,
    #[serde(default, rename = "@reroute")]
    pub reroute: Option<BoolType>,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@via")]
    pub via: Option<String>,
    #[serde(default, rename = "@type")]
    pub type_: Option<String>,
    #[serde(rename = "@depart")]
    pub depart: DepartType,
    #[serde(default, rename = "@color")]
    pub color: Option<ColorType>,
    #[serde(default, rename = "@departLane")]
    pub depart_lane: Option<DepartLaneType>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@departSpeed")]
    pub depart_speed: Option<DepartSpeedType>,
    #[serde(default, rename = "@departEdge")]
    pub depart_edge: Option<RouteIndexType>,
    #[serde(default, rename = "@arrivalEdge")]
    pub arrival_edge: Option<RouteIndexType>,
    #[serde(default, rename = "@arrivalLane")]
    pub arrival_lane: Option<ArrivalLaneType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@arrivalSpeed")]
    pub arrival_speed: Option<ArrivalSpeedType>,
    #[serde(default, rename = "@departPosLat")]
    pub depart_pos_lat: Option<DepartPosLatType>,
    #[serde(default, rename = "@arrivalPosLat")]
    pub arrival_pos_lat: Option<ArrivalPosLatType>,
    #[serde(default, rename = "@arrival")]
    pub arrival: Option<TimeType>,
    #[serde(default, rename = "@routeLength")]
    pub route_length: Option<f32>,
    #[serde(default, rename = "@line")]
    pub line: Option<String>,
    #[serde(default, rename = "@personNumber")]
    pub person_number: Option<u32>,
    #[serde(default, rename = "@containerNumber")]
    pub container_number: Option<u32>,
    #[serde(default, rename = "@speedFactor")]
    pub speed_factor: Option<f32>,
    #[serde(default, rename = "@insertionChecks")]
    pub insertion_checks: Option<String>,
    #[serde(default, rename = "@parkingBadges")]
    pub parking_badges: Option<String>,
    #[serde(rename = "#content")]
    pub content: Vec<VehicleTypeContent>,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum VehicleTypeContent {
    #[serde(rename = "param")]
    Param(ParamType),
    #[serde(rename = "route")]
    Route(VehicleRouteType),
    #[serde(rename = "routeDistribution")]
    RouteDistribution(VehicleRouteDistributionType),
    #[serde(rename = "stop")]
    Stop(StopType),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerBaseTypeTransportElementType {
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@fromJunction")]
    pub from_junction: Option<String>,
    #[serde(default, rename = "@toJunction")]
    pub to_junction: Option<String>,
    #[serde(default, rename = "@busStop")]
    pub bus_stop: Option<String>,
    #[serde(default, rename = "@trainStop")]
    pub train_stop: Option<String>,
    #[serde(default, rename = "@parkingArea")]
    pub parking_area: Option<String>,
    #[serde(default, rename = "@containerStop")]
    pub container_stop: Option<String>,
    #[serde(default, rename = "@chargingStation")]
    pub charging_station: Option<String>,
    #[serde(default, rename = "@lines")]
    pub lines: Option<String>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<String>,
    #[serde(default, rename = "@cost")]
    pub cost: Option<String>,
    #[serde(default, rename = "@intended")]
    pub intended: Option<String>,
    #[serde(default, rename = "@depart")]
    pub depart: Option<TimeType>,
    #[serde(default, rename = "@routeLength")]
    pub route_length: Option<String>,
    #[serde(default, rename = "@group")]
    pub group: Option<String>,
    #[serde(default, rename = "@vehicle")]
    pub vehicle: Option<String>,
    #[serde(default, rename = "@started")]
    pub started: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "@ended")]
    pub ended: Option<TimeTypeWithErrorValueType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerBaseTypeTranshipElementType {
    #[serde(default, rename = "@edges")]
    pub edges: Option<String>,
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@fromJunction")]
    pub from_junction: Option<String>,
    #[serde(default, rename = "@toJunction")]
    pub to_junction: Option<String>,
    #[serde(default, rename = "@busStop")]
    pub bus_stop: Option<String>,
    #[serde(default, rename = "@trainStop")]
    pub train_stop: Option<String>,
    #[serde(default, rename = "@parkingArea")]
    pub parking_area: Option<String>,
    #[serde(default, rename = "@containerStop")]
    pub container_stop: Option<String>,
    #[serde(default, rename = "@chargingStation")]
    pub charging_station: Option<String>,
    #[serde(default, rename = "@speed")]
    pub speed: Option<f32>,
    #[serde(default, rename = "@duration")]
    pub duration: Option<f32>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<String>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<String>,
    #[serde(default, rename = "@started")]
    pub started: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "@ended")]
    pub ended: Option<TimeTypeWithErrorValueType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonBaseTypePersonTripElementType {
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@fromXY")]
    pub from_xy: Option<String>,
    #[serde(default, rename = "@toXY")]
    pub to_xy: Option<String>,
    #[serde(default, rename = "@fromLonLat")]
    pub from_lon_lat: Option<String>,
    #[serde(default, rename = "@toLonLat")]
    pub to_lon_lat: Option<String>,
    #[serde(default, rename = "@fromJunction")]
    pub from_junction: Option<String>,
    #[serde(default, rename = "@toJunction")]
    pub to_junction: Option<String>,
    #[serde(default, rename = "@viaJunctions")]
    pub via_junctions: Option<String>,
    #[serde(default, rename = "@busStop")]
    pub bus_stop: Option<String>,
    #[serde(default, rename = "@trainStop")]
    pub train_stop: Option<String>,
    #[serde(default, rename = "@parkingArea")]
    pub parking_area: Option<String>,
    #[serde(default, rename = "@containerStop")]
    pub container_stop: Option<String>,
    #[serde(default, rename = "@chargingStation")]
    pub charging_station: Option<String>,
    #[serde(default, rename = "@modes")]
    pub modes: Option<String>,
    #[serde(default, rename = "@vTypes")]
    pub v_types: Option<String>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@departPosLat")]
    pub depart_pos_lat: Option<DepartPosLatType>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@walkFactor")]
    pub walk_factor: Option<f32>,
    #[serde(default, rename = "@costs")]
    pub costs: Option<String>,
    #[serde(default, rename = "@group")]
    pub group: Option<String>,
    #[serde(default, rename = "@lines")]
    pub lines: Option<String>,
    #[serde(default, rename = "param")]
    pub param: Vec<ParamType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonBaseTypeRideElementType {
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@fromLonLat")]
    pub from_lon_lat: Option<String>,
    #[serde(default, rename = "@toLonLat")]
    pub to_lon_lat: Option<String>,
    #[serde(default, rename = "@fromXY")]
    pub from_xy: Option<String>,
    #[serde(default, rename = "@toXY")]
    pub to_xy: Option<String>,
    #[serde(default, rename = "@busStop")]
    pub bus_stop: Option<String>,
    #[serde(default, rename = "@trainStop")]
    pub train_stop: Option<String>,
    #[serde(default, rename = "@parkingArea")]
    pub parking_area: Option<String>,
    #[serde(default, rename = "@containerStop")]
    pub container_stop: Option<String>,
    #[serde(default, rename = "@chargingStation")]
    pub charging_station: Option<String>,
    #[serde(default, rename = "@lines")]
    pub lines: Option<String>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<String>,
    #[serde(default, rename = "@cost")]
    pub cost: Option<String>,
    #[serde(default, rename = "@intended")]
    pub intended: Option<String>,
    #[serde(default, rename = "@depart")]
    pub depart: Option<TimeType>,
    #[serde(default, rename = "@routeLength")]
    pub route_length: Option<String>,
    #[serde(default, rename = "@group")]
    pub group: Option<String>,
    #[serde(default, rename = "@vehicle")]
    pub vehicle: Option<String>,
    #[serde(default, rename = "@started")]
    pub started: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "@ended")]
    pub ended: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "param")]
    pub param: Vec<ParamType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PersonBaseTypeWalkElementType {
    #[serde(default, rename = "@route")]
    pub route: Option<String>,
    #[serde(default, rename = "@edges")]
    pub edges: Option<String>,
    #[serde(default, rename = "@from")]
    pub from: Option<String>,
    #[serde(default, rename = "@to")]
    pub to: Option<String>,
    #[serde(default, rename = "@fromTaz")]
    pub from_taz: Option<String>,
    #[serde(default, rename = "@toTaz")]
    pub to_taz: Option<String>,
    #[serde(default, rename = "@fromXY")]
    pub from_xy: Option<String>,
    #[serde(default, rename = "@toXY")]
    pub to_xy: Option<String>,
    #[serde(default, rename = "@fromLonLat")]
    pub from_lon_lat: Option<String>,
    #[serde(default, rename = "@toLonLat")]
    pub to_lon_lat: Option<String>,
    #[serde(default, rename = "@fromJunction")]
    pub from_junction: Option<String>,
    #[serde(default, rename = "@toJunction")]
    pub to_junction: Option<String>,
    #[serde(default, rename = "@viaJunctions")]
    pub via_junctions: Option<String>,
    #[serde(default, rename = "@busStop")]
    pub bus_stop: Option<String>,
    #[serde(default, rename = "@trainStop")]
    pub train_stop: Option<String>,
    #[serde(default, rename = "@parkingArea")]
    pub parking_area: Option<String>,
    #[serde(default, rename = "@containerStop")]
    pub container_stop: Option<String>,
    #[serde(default, rename = "@chargingStation")]
    pub charging_station: Option<String>,
    #[serde(default, rename = "@speed")]
    pub speed: Option<f32>,
    #[serde(default, rename = "@duration")]
    pub duration: Option<f32>,
    #[serde(default, rename = "@departPos")]
    pub depart_pos: Option<DepartPosType>,
    #[serde(default, rename = "@departPosLat")]
    pub depart_pos_lat: Option<DepartPosLatType>,
    #[serde(default, rename = "@departLane")]
    pub depart_lane: Option<u32>,
    #[serde(default, rename = "@arrivalPos")]
    pub arrival_pos: Option<ArrivalPosType>,
    #[serde(default, rename = "@cost")]
    pub cost: Option<String>,
    #[serde(default, rename = "@routeLength")]
    pub route_length: Option<String>,
    #[serde(default, rename = "@exitTimes")]
    pub exit_times: Option<String>,
    #[serde(default, rename = "@started")]
    pub started: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "@ended")]
    pub ended: Option<TimeTypeWithErrorValueType>,
    #[serde(default, rename = "param")]
    pub param: Vec<ParamType>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StopTypeParkingType {
    #[serde(rename = "#text")]
    pub value: StopTypeParkingValue,
}
impl From<StopTypeParkingValue> for StopTypeParkingType {
    fn from(value: StopTypeParkingValue) -> Self {
        Self { value }
    }
}
impl From<StopTypeParkingType> for StopTypeParkingValue {
    fn from(value: StopTypeParkingType) -> Self {
        value.value
    }
}
impl Deref for StopTypeParkingType {
    type Target = StopTypeParkingValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for StopTypeParkingType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum StopTypeParkingValue {
    #[serde(rename = "true")]
    SmallTrue,
    #[serde(rename = "false")]
    SmallFalse,
    #[serde(rename = "True")]
    True,
    #[serde(rename = "False")]
    False,
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "on")]
    On,
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "1")]
    _1,
    #[serde(rename = "0")]
    _0,
    #[serde(rename = "x")]
    X,
    #[serde(rename = "opportunistic")]
    Opportunistic,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TypeTypeSpreadType {
    #[serde(rename = "#text")]
    pub value: TypeTypeSpreadTypeValue,
}
impl From<TypeTypeSpreadTypeValue> for TypeTypeSpreadType {
    fn from(value: TypeTypeSpreadTypeValue) -> Self {
        Self { value }
    }
}
impl From<TypeTypeSpreadType> for TypeTypeSpreadTypeValue {
    fn from(value: TypeTypeSpreadType) -> Self {
        value.value
    }
}
impl Deref for TypeTypeSpreadType {
    type Target = TypeTypeSpreadTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for TypeTypeSpreadType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TypeTypeSpreadTypeValue {
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "roadCenter")]
    RoadCenter,
    #[serde(rename = "right")]
    Right,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VTypeTypeImpatienceType {
    #[serde(rename = "#text")]
    pub value: VTypeTypeImpatienceValue,
}
impl From<VTypeTypeImpatienceValue> for VTypeTypeImpatienceType {
    fn from(value: VTypeTypeImpatienceValue) -> Self {
        Self { value }
    }
}
impl From<VTypeTypeImpatienceType> for VTypeTypeImpatienceValue {
    fn from(value: VTypeTypeImpatienceType) -> Self {
        value.value
    }
}
impl Deref for VTypeTypeImpatienceType {
    type Target = VTypeTypeImpatienceValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for VTypeTypeImpatienceType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum VTypeTypeImpatienceValue {
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "off")]
    Off,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VTypeTypeTrainType {
    #[serde(rename = "#text")]
    pub value: VTypeTypeTrainTypeValue,
}
impl From<VTypeTypeTrainTypeValue> for VTypeTypeTrainType {
    fn from(value: VTypeTypeTrainTypeValue) -> Self {
        Self { value }
    }
}
impl From<VTypeTypeTrainType> for VTypeTypeTrainTypeValue {
    fn from(value: VTypeTypeTrainType) -> Self {
        value.value
    }
}
impl Deref for VTypeTypeTrainType {
    type Target = VTypeTypeTrainTypeValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for VTypeTypeTrainType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum VTypeTypeTrainTypeValue {
    #[serde(rename = "RB425")]
    Rb425,
    #[serde(rename = "NGT400")]
    Ngt400,
    #[serde(rename = "NGT400_16")]
    Ngt40016,
    #[serde(rename = "ICE1")]
    Ice1,
    #[serde(rename = "ICE3")]
    Ice3,
    #[serde(rename = "REDosto7")]
    ReDosto7,
    #[serde(rename = "RB628")]
    Rb628,
    #[serde(rename = "Freight")]
    Freight,
    #[serde(rename = "MireoPlusB")]
    MireoPlusB,
    #[serde(rename = "MireoPlusH")]
    MireoPlusH,
    #[serde(rename = "custom")]
    Custom,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VTypeTypeLaneChangeModelType {
    #[serde(rename = "#text")]
    pub value: VTypeTypeLaneChangeModelValue,
}
impl From<VTypeTypeLaneChangeModelValue> for VTypeTypeLaneChangeModelType {
    fn from(value: VTypeTypeLaneChangeModelValue) -> Self {
        Self { value }
    }
}
impl From<VTypeTypeLaneChangeModelType> for VTypeTypeLaneChangeModelValue {
    fn from(value: VTypeTypeLaneChangeModelType) -> Self {
        value.value
    }
}
impl Deref for VTypeTypeLaneChangeModelType {
    type Target = VTypeTypeLaneChangeModelValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for VTypeTypeLaneChangeModelType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum VTypeTypeLaneChangeModelValue {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "DK2008")]
    Dk2008,
    #[serde(rename = "LC2013")]
    Lc2013,
    #[serde(rename = "LC2013_CC")]
    Lc2013Cc,
    #[serde(rename = "SL2015")]
    Sl2015,
}
pub mod xs {
    use serde_derive::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct EntitiesType(pub Vec<String>);
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct EntityType(pub Vec<String>);
    pub type IdType = String;
    pub type IdrefType = String;
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct IdrefsType(pub Vec<String>);
    pub type NcNameType = String;
    pub type NmtokenType = String;
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct NmtokensType(pub Vec<String>);
    pub type NotationType = String;
    pub type NameType = String;
    pub type QNameType = String;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AnyType;
    pub type AnyUriType = String;
    pub type Base64BinaryType = String;
    pub type BooleanType = bool;
    pub type ByteType = i8;
    pub type DateType = String;
    pub type DateTimeType = String;
    pub type DecimalType = f64;
    pub type DoubleType = f64;
    pub type DurationType = String;
    pub type FloatType = f32;
    pub type GDayType = String;
    pub type GMonthType = String;
    pub type GMonthDayType = String;
    pub type GYearType = String;
    pub type GYearMonthType = String;
    pub type HexBinaryType = String;
    pub type IntType = i32;
    pub type IntegerType = i32;
    pub type LanguageType = String;
    pub type LongType = i64;
    pub type NegativeIntegerType = i32;
    pub type NonNegativeIntegerType = u32;
    pub type NonPositiveIntegerType = i32;
    pub type NormalizedStringType = String;
    pub type PositiveIntegerType = u32;
    pub type ShortType = i16;
    pub type StringType = String;
    pub type TimeType = String;
    pub type TokenType = String;
    pub type UnsignedByteType = u8;
    pub type UnsignedIntType = u32;
    pub type UnsignedLongType = u64;
    pub type UnsignedShortType = u16;
}
