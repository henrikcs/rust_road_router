/// This file has been automatically generated using xsd-parser: https://github.com/Bergmann89/xsd-parser
/// Note that some parts have been adapted where "=", "-", and values like "true" and "True" in the same enum were problematic.
use core::ops::{Deref, DerefMut};
use serde_derive::{Deserialize, Serialize};
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
pub type DetectorIdType = String;
#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeLaneDataType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@numEdges")]
    pub num_edges: Option<i32>,
    #[serde(default, rename = "@sampledSeconds")]
    pub sampled_seconds: Option<f32>,
    #[serde(default, rename = "@traveltime")]
    pub traveltime: Option<f32>,
    #[serde(default, rename = "@overlapTraveltime")]
    pub overlap_traveltime: Option<f32>,
    #[serde(default, rename = "@density")]
    pub density: Option<f32>,
    #[serde(default, rename = "@laneDensity")]
    pub lane_density: Option<f32>,
    #[serde(default, rename = "@occupancy")]
    pub occupancy: Option<f32>,
    #[serde(default, rename = "@waitingTime")]
    pub waiting_time: Option<f32>,
    #[serde(default, rename = "@timeLoss")]
    pub time_loss: Option<String>,
    #[serde(default, rename = "@speed")]
    pub speed: Option<f32>,
    #[serde(default, rename = "@speedRelative")]
    pub speed_relative: Option<f32>,
    #[serde(default, rename = "@departed")]
    pub departed: Option<i32>,
    #[serde(default, rename = "@arrived")]
    pub arrived: Option<i32>,
    #[serde(default, rename = "@entered")]
    pub entered: Option<f32>,
    #[serde(default, rename = "@left")]
    pub left: Option<i32>,
    #[serde(default, rename = "@laneChangedFrom")]
    pub lane_changed_from: Option<i32>,
    #[serde(default, rename = "@laneChangedTo")]
    pub lane_changed_to: Option<i32>,
    #[serde(default, rename = "@vaporized")]
    pub vaporized: Option<i32>,
    #[serde(default, rename = "@vaporizedOnNextEdge")]
    pub vaporized_on_next_edge: Option<i32>,
    #[serde(default, rename = "@teleported")]
    pub teleported: Option<i32>,
    #[serde(default, rename = "@CO_abs")]
    pub co_abs: Option<f32>,
    #[serde(default, rename = "@CO2_abs")]
    pub co_2_abs: Option<f32>,
    #[serde(default, rename = "@HC_abs")]
    pub hc_abs: Option<f32>,
    #[serde(default, rename = "@PMx_abs")]
    pub p_mx_abs: Option<f32>,
    #[serde(default, rename = "@NOx_abs")]
    pub n_ox_abs: Option<f32>,
    #[serde(default, rename = "@fuel_abs")]
    pub fuel_abs: Option<f32>,
    #[serde(default, rename = "@electricity_abs")]
    pub electricity_abs: Option<String>,
    #[serde(default, rename = "@CO_normed")]
    pub co_normed: Option<f32>,
    #[serde(default, rename = "@CO2_normed")]
    pub co_2_normed: Option<f32>,
    #[serde(default, rename = "@HC_normed")]
    pub hc_normed: Option<f32>,
    #[serde(default, rename = "@PMx_normed")]
    pub p_mx_normed: Option<f32>,
    #[serde(default, rename = "@NOx_normed")]
    pub n_ox_normed: Option<f32>,
    #[serde(default, rename = "@fuel_normed")]
    pub fuel_normed: Option<f32>,
    #[serde(default, rename = "@electricity_normed")]
    pub electricity_normed: Option<String>,
    #[serde(default, rename = "@CO_perVeh")]
    pub co_per_veh: Option<f32>,
    #[serde(default, rename = "@CO2_perVeh")]
    pub co_2_per_veh: Option<f32>,
    #[serde(default, rename = "@HC_perVeh")]
    pub hc_per_veh: Option<f32>,
    #[serde(default, rename = "@PMx_perVeh")]
    pub p_mx_per_veh: Option<f32>,
    #[serde(default, rename = "@NOx_perVeh")]
    pub n_ox_per_veh: Option<f32>,
    #[serde(default, rename = "@fuel_perVeh")]
    pub fuel_per_veh: Option<f32>,
    #[serde(default, rename = "@electricity_perVeh")]
    pub electricity_per_veh: Option<String>,
    #[serde(default, rename = "@noise")]
    pub noise: Option<f32>,
}
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
pub struct IntervalType {
    #[serde(rename = "@begin")]
    pub begin: TimeType,
    #[serde(rename = "@end")]
    pub end: TimeType,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "edge")]
    pub edge: Vec<IntervalTypeEdgeElementType>,
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
pub type Meandata = MeandataElementType;
#[derive(Debug, Serialize, Deserialize)]
pub struct MeandataElementType {
    #[serde(default, rename = "interval")]
    pub interval: Vec<IntervalType>,
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
pub struct IntervalTypeEdgeElementType {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default, rename = "@numEdges")]
    pub num_edges: Option<i32>,
    #[serde(default, rename = "@sampledSeconds")]
    pub sampled_seconds: Option<f32>,
    #[serde(default, rename = "@traveltime")]
    pub traveltime: Option<f32>,
    #[serde(default, rename = "@overlapTraveltime")]
    pub overlap_traveltime: Option<f32>,
    #[serde(default, rename = "@density")]
    pub density: Option<f32>,
    #[serde(default, rename = "@laneDensity")]
    pub lane_density: Option<f32>,
    #[serde(default, rename = "@occupancy")]
    pub occupancy: Option<f32>,
    #[serde(default, rename = "@waitingTime")]
    pub waiting_time: Option<f32>,
    #[serde(default, rename = "@timeLoss")]
    pub time_loss: Option<String>,
    #[serde(default, rename = "@speed")]
    pub speed: Option<f32>,
    #[serde(default, rename = "@speedRelative")]
    pub speed_relative: Option<f32>,
    #[serde(default, rename = "@departed")]
    pub departed: Option<i32>,
    #[serde(default, rename = "@arrived")]
    pub arrived: Option<i32>,
    #[serde(default, rename = "@entered")]
    pub entered: Option<f32>,
    #[serde(default, rename = "@left")]
    pub left: Option<i32>,
    #[serde(default, rename = "@laneChangedFrom")]
    pub lane_changed_from: Option<i32>,
    #[serde(default, rename = "@laneChangedTo")]
    pub lane_changed_to: Option<i32>,
    #[serde(default, rename = "@vaporized")]
    pub vaporized: Option<i32>,
    #[serde(default, rename = "@vaporizedOnNextEdge")]
    pub vaporized_on_next_edge: Option<i32>,
    #[serde(default, rename = "@teleported")]
    pub teleported: Option<i32>,
    #[serde(default, rename = "@CO_abs")]
    pub co_abs: Option<f32>,
    #[serde(default, rename = "@CO2_abs")]
    pub co_2_abs: Option<f32>,
    #[serde(default, rename = "@HC_abs")]
    pub hc_abs: Option<f32>,
    #[serde(default, rename = "@PMx_abs")]
    pub p_mx_abs: Option<f32>,
    #[serde(default, rename = "@NOx_abs")]
    pub n_ox_abs: Option<f32>,
    #[serde(default, rename = "@fuel_abs")]
    pub fuel_abs: Option<f32>,
    #[serde(default, rename = "@electricity_abs")]
    pub electricity_abs: Option<String>,
    #[serde(default, rename = "@CO_normed")]
    pub co_normed: Option<f32>,
    #[serde(default, rename = "@CO2_normed")]
    pub co_2_normed: Option<f32>,
    #[serde(default, rename = "@HC_normed")]
    pub hc_normed: Option<f32>,
    #[serde(default, rename = "@PMx_normed")]
    pub p_mx_normed: Option<f32>,
    #[serde(default, rename = "@NOx_normed")]
    pub n_ox_normed: Option<f32>,
    #[serde(default, rename = "@fuel_normed")]
    pub fuel_normed: Option<f32>,
    #[serde(default, rename = "@electricity_normed")]
    pub electricity_normed: Option<String>,
    #[serde(default, rename = "@CO_perVeh")]
    pub co_per_veh: Option<f32>,
    #[serde(default, rename = "@CO2_perVeh")]
    pub co_2_per_veh: Option<f32>,
    #[serde(default, rename = "@HC_perVeh")]
    pub hc_per_veh: Option<f32>,
    #[serde(default, rename = "@PMx_perVeh")]
    pub p_mx_per_veh: Option<f32>,
    #[serde(default, rename = "@NOx_perVeh")]
    pub n_ox_per_veh: Option<f32>,
    #[serde(default, rename = "@fuel_perVeh")]
    pub fuel_per_veh: Option<f32>,
    #[serde(default, rename = "@electricity_perVeh")]
    pub electricity_per_veh: Option<String>,
    #[serde(default, rename = "@noise")]
    pub noise: Option<f32>,
    #[serde(default, rename = "lane")]
    pub lane: Vec<EdgeLaneDataType>,
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
