// schema.rs - Schema of data sources
// Sasaki, Naoki <nsasaki@sal.co.jp> January 29, 2023
//

use std::sync::Arc;

use datafusion::arrow::{self, datatypes::SchemaRef};
use serde::Deserialize;
use serde_derive::Serialize;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DataType {
    Unknown,
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Integer, // alias as Int64
    Float16,
    Float32,
    Float64,
    Float, // alias as Float64
    Decimal128(DecimalType),
    Decimal256(DecimalType),
    Decimal(DecimalType), // alias as Decimal256
    Timestamp(TimestampType),
    Date32,           // elapsed time since 1 January 1970 in days
    Date64,           // elapsed time since 00:00:00.000 on 1 January 1970 in milliseconds
    Date,             // alias as Date64
    Time32(TimeType), // elapsed time since midnight in seconds or milliseconds
    Time64(TimeType), // elapsed time since midnight in microseconds or nanoseconds
    Time(TimeType),   // alias as Time32
    Duration(DurationType),
    Interval(IntervalType),
    String, // variable length string in Unicode with UTF-8 encoding
    List(Box<DataType>),
    LargeList(Box<DataType>),
    Map(Box<DataType>, Box<DataType>),
    Struct(Vec<(String, DataType)>),
    Union(UnionType),
}

impl DataType {
    fn to_arrow_data_type(&self) -> arrow::datatypes::DataType {
        match self {
            DataType::Boolean => arrow::datatypes::DataType::Boolean,
            DataType::Int8 => arrow::datatypes::DataType::Int8,
            DataType::Int16 => arrow::datatypes::DataType::Int16,
            DataType::Int32 => arrow::datatypes::DataType::Int32,
            DataType::Int64 | DataType::Integer => arrow::datatypes::DataType::Int64,
            DataType::UInt8 => arrow::datatypes::DataType::UInt8,
            DataType::UInt16 => arrow::datatypes::DataType::UInt16,
            DataType::UInt32 => arrow::datatypes::DataType::UInt32,
            DataType::UInt64 => arrow::datatypes::DataType::UInt64,
            DataType::Float16 => arrow::datatypes::DataType::Float16,
            DataType::Float32 => arrow::datatypes::DataType::Float32,
            DataType::Float64 | DataType::Float => arrow::datatypes::DataType::Float64,
            DataType::Decimal128(decimal_type) => {
                arrow::datatypes::DataType::Decimal128(decimal_type.precision, decimal_type.scale)
            }
            DataType::Decimal(decimal_type) | DataType::Decimal256(decimal_type) => {
                arrow::datatypes::DataType::Decimal256(decimal_type.precision, decimal_type.scale)
            }
            DataType::Timestamp(timestamp_type) => timestamp_type.clone().into_arrow_timestamp(),
            DataType::Date32 => arrow::datatypes::DataType::Date32,
            DataType::Date64 | DataType::Date => arrow::datatypes::DataType::Date64,
            DataType::Time32(time_type) | DataType::Time(time_type) => {
                arrow::datatypes::DataType::Time32(time_type.unit.to_arrow_time_unit())
            }
            DataType::Time64(time_type) => {
                arrow::datatypes::DataType::Time64(time_type.unit.to_arrow_time_unit())
            }
            DataType::Duration(duration_type) => duration_type.clone().into_arrow_duration(),
            DataType::Interval(interval_type) => interval_type.clone().into_arrow_interval(),
            DataType::String => arrow::datatypes::DataType::Utf8,
            DataType::List(child_type) => {
                arrow::datatypes::DataType::List(arrow::datatypes::FieldRef::from(
                    arrow::datatypes::Field::new("item", child_type.to_arrow_data_type(), true),
                ))
            }
            DataType::LargeList(child_type) => {
                arrow::datatypes::DataType::LargeList(arrow::datatypes::FieldRef::from(
                    arrow::datatypes::Field::new("item", child_type.to_arrow_data_type(), true),
                ))
            }
            DataType::Map(key_type, value_type) => arrow::datatypes::DataType::Map(
                arrow::datatypes::FieldRef::from(arrow::datatypes::Field::new(
                    "entry",
                    arrow::datatypes::DataType::Struct(arrow::datatypes::Fields::from(vec![
                        arrow::datatypes::Field::new("key", key_type.to_arrow_data_type(), false),
                        arrow::datatypes::Field::new(
                            "value",
                            value_type.to_arrow_data_type(),
                            true,
                        ),
                    ])),
                    false,
                )),
                false,
            ),
            DataType::Struct(fields) => arrow::datatypes::DataType::Struct(
                fields
                    .iter()
                    .map(|(name, data_type)| {
                        arrow::datatypes::Field::new(name, data_type.to_arrow_data_type(), true)
                    })
                    .collect(),
            ),
            DataType::Union(union_type) => {
                let type_ids = union_type
                    .types
                    .iter()
                    .map(|(type_id, _)| *type_id)
                    .collect::<Vec<i8>>();
                let fields = union_type
                    .types
                    .iter()
                    .map(|(_, my_data_type)| {
                        Arc::new(arrow::datatypes::Field::new(
                            "",
                            my_data_type.to_arrow_data_type(),
                            true,
                        )) as arrow::datatypes::FieldRef
                    })
                    .collect::<Vec<arrow::datatypes::FieldRef>>();

                arrow::datatypes::DataType::Union(
                    arrow::datatypes::UnionFields::new(type_ids, fields),
                    UnionMode::to_arrow_union_mode(&union_type.mode),
                )
            }
            DataType::Unknown => arrow::datatypes::DataType::Binary,
        }
    }

    fn from_arrow_data_type(arrow_data_type: &arrow::datatypes::DataType) -> DataType {
        match arrow_data_type {
            arrow::datatypes::DataType::Boolean => DataType::Boolean,
            arrow::datatypes::DataType::Int8 => DataType::Int8,
            arrow::datatypes::DataType::Int16 => DataType::Int16,
            arrow::datatypes::DataType::Int32 => DataType::Int32,
            arrow::datatypes::DataType::Int64 => DataType::Int64,
            arrow::datatypes::DataType::UInt8 => DataType::UInt8,
            arrow::datatypes::DataType::UInt16 => DataType::UInt16,
            arrow::datatypes::DataType::UInt32 => DataType::UInt32,
            arrow::datatypes::DataType::UInt64 => DataType::UInt64,
            arrow::datatypes::DataType::Float16 => DataType::Float16,
            arrow::datatypes::DataType::Float32 => DataType::Float32,
            arrow::datatypes::DataType::Float64 => DataType::Float64,
            arrow::datatypes::DataType::Decimal128(precision, scale) => {
                DataType::Decimal128(DecimalType {
                    precision: *precision,
                    scale: *scale,
                })
            }
            arrow::datatypes::DataType::Decimal256(precision, scale) => {
                DataType::Decimal256(DecimalType {
                    precision: *precision,
                    scale: *scale,
                })
            }
            arrow::datatypes::DataType::Timestamp(unit, tz) => DataType::Timestamp(TimestampType {
                unit: TimeUnit::from_arrow_time_unit(unit),
                timezone: tz.as_ref().map(std::string::ToString::to_string),
            }),
            arrow::datatypes::DataType::Date32 => DataType::Date32,
            arrow::datatypes::DataType::Date64 => DataType::Date64,
            arrow::datatypes::DataType::Time32(unit) => DataType::Time32(TimeType {
                unit: TimeUnit::from_arrow_time_unit(unit),
            }),
            arrow::datatypes::DataType::Time64(unit) => DataType::Time64(TimeType {
                unit: TimeUnit::from_arrow_time_unit(unit),
            }),
            arrow::datatypes::DataType::Duration(unit) => DataType::Duration(DurationType {
                unit: TimeUnit::from_arrow_time_unit(unit),
            }),
            arrow::datatypes::DataType::Interval(unit) => DataType::Interval(IntervalType {
                unit: IntervalUnit::from_arrow_interval_unit(unit),
            }),
            arrow::datatypes::DataType::Utf8 => DataType::String,
            arrow::datatypes::DataType::List(field) => {
                DataType::List(Box::new(Self::from_arrow_data_type(field.data_type())))
            }
            arrow::datatypes::DataType::LargeList(field) => {
                DataType::LargeList(Box::new(Self::from_arrow_data_type(field.data_type())))
            }
            arrow::datatypes::DataType::Map(field, _keys_sorted) => {
                let arrow::datatypes::DataType::Struct(fields) = field.data_type() else {
                    // TODO: error handling
                    panic!("Expected DataType::Struct but found something else")
                };
                let key_type = Self::from_arrow_data_type(fields[0].data_type());
                let value_type = Self::from_arrow_data_type(fields[1].data_type());
                DataType::Map(Box::new(key_type), Box::new(value_type))
            }
            arrow::datatypes::DataType::Struct(fields) => DataType::Struct(
                fields
                    .iter()
                    .map(|field| {
                        (
                            field.name().to_string(),
                            Self::from_arrow_data_type(field.data_type()),
                        )
                    })
                    .collect::<Vec<(String, DataType)>>(),
            ),
            arrow::datatypes::DataType::Union(union_fields, mode) => {
                let types = union_fields
                    .iter()
                    .map(|(type_id, field_ref)| {
                        (type_id, Self::from_arrow_data_type(field_ref.data_type()))
                    })
                    .collect();
                DataType::Union(UnionType {
                    types,
                    mode: UnionMode::from_arrow_union_mode(*mode),
                })
            }
            _ => DataType::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimestampType {
    unit: TimeUnit,
    timezone: Option<String>,
}

impl TimestampType {
    fn into_arrow_timestamp(self) -> arrow::datatypes::DataType {
        arrow::datatypes::DataType::Timestamp(
            self.unit.to_arrow_time_unit(),
            self.timezone.map(|tz| Arc::from(tz.as_str())),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DurationType {
    unit: TimeUnit,
}

impl DurationType {
    fn into_arrow_duration(self) -> arrow::datatypes::DataType {
        arrow::datatypes::DataType::Duration(self.unit.to_arrow_time_unit())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeType {
    unit: TimeUnit,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TimeUnit {
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl TimeUnit {
    fn to_arrow_time_unit(&self) -> arrow::datatypes::TimeUnit {
        match self {
            Self::Second => arrow::datatypes::TimeUnit::Second,
            Self::Millisecond => arrow::datatypes::TimeUnit::Millisecond,
            Self::Microsecond => arrow::datatypes::TimeUnit::Microsecond,
            Self::Nanosecond => arrow::datatypes::TimeUnit::Nanosecond,
        }
    }

    fn from_arrow_time_unit(time_unit: &arrow::datatypes::TimeUnit) -> Self {
        match time_unit {
            arrow::datatypes::TimeUnit::Second => Self::Second,
            arrow::datatypes::TimeUnit::Millisecond => Self::Millisecond,
            arrow::datatypes::TimeUnit::Microsecond => Self::Microsecond,
            arrow::datatypes::TimeUnit::Nanosecond => Self::Nanosecond,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntervalType {
    unit: IntervalUnit,
}

impl IntervalType {
    fn into_arrow_interval(self) -> arrow::datatypes::DataType {
        arrow::datatypes::DataType::Interval(self.unit.to_arrow_interval_unit())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum IntervalUnit {
    YearMonth,
    DayTime,
    MonthDayNano,
}

impl IntervalUnit {
    fn to_arrow_interval_unit(&self) -> arrow::datatypes::IntervalUnit {
        match self {
            Self::YearMonth => arrow::datatypes::IntervalUnit::YearMonth,
            Self::DayTime => arrow::datatypes::IntervalUnit::DayTime,
            Self::MonthDayNano => arrow::datatypes::IntervalUnit::MonthDayNano,
        }
    }

    fn from_arrow_interval_unit(interval_unit: &arrow::datatypes::IntervalUnit) -> Self {
        match interval_unit {
            arrow::datatypes::IntervalUnit::YearMonth => Self::YearMonth,
            arrow::datatypes::IntervalUnit::DayTime => Self::DayTime,
            arrow::datatypes::IntervalUnit::MonthDayNano => Self::MonthDayNano,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DecimalType {
    precision: u8,
    scale: i8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapType {
    key: Box<DataType>,
    value: Box<DataType>,
    ordered: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnionType {
    types: Vec<(i8, DataType)>,
    mode: UnionMode,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UnionMode {
    Sparse,
    Dense,
}

impl UnionMode {
    fn to_arrow_union_mode(&self) -> arrow::datatypes::UnionMode {
        match self {
            UnionMode::Sparse => arrow::datatypes::UnionMode::Sparse,
            UnionMode::Dense => arrow::datatypes::UnionMode::Dense,
        }
    }

    fn from_arrow_union_mode(mode: arrow::datatypes::UnionMode) -> Self {
        match mode {
            arrow::datatypes::UnionMode::Sparse => UnionMode::Sparse,
            arrow::datatypes::UnionMode::Dense => UnionMode::Dense,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Field {
    pub name: String,
    #[serde(rename = "dataType")]
    pub data_type: DataType,
    pub nullable: Option<bool>,
}

impl Field {
    #[allow(dead_code)]
    fn to_arrow_field(&self) -> Arc<arrow::datatypes::Field> {
        Arc::new(arrow::datatypes::Field::new(
            self.name.clone(),
            self.data_type.to_arrow_data_type(),
            self.nullable.unwrap_or(true),
        ))
    }

    pub fn from_arrow_field(field: &arrow::datatypes::Field) -> Self {
        Self {
            name: field.name().clone(),
            data_type: DataType::from_arrow_data_type(field.data_type()),
            nullable: Some(field.is_nullable()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
#[allow(clippy::module_name_repetitions)]
pub struct DataSourceSchema {
    pub fields: Vec<Field>,
}

impl DataSourceSchema {
    pub fn to_arrow_schema(&self) -> arrow::datatypes::Schema {
        let mut schema_fields = Vec::<arrow::datatypes::Field>::new();

        for field in &self.fields {
            schema_fields.push(arrow::datatypes::Field::new(
                field.name.clone(),
                field.data_type.to_arrow_data_type(),
                field.nullable.unwrap_or(true),
            ));
        }

        arrow::datatypes::Schema::new(schema_fields)
    }

    pub fn from_arrow_schema(schema: &SchemaRef) -> Self {
        let mut fields = Vec::<Field>::new();

        for field in &schema.fields {
            fields.push(Field::from_arrow_field(field));
        }

        Self { fields }
    }
}
