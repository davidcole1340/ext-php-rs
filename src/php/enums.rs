/// Valid data types for PHP.
#[derive(Clone, Copy)]
pub enum DataType {
    Undef = 0,
    Null = 1,
    False = 2,
    True = 3,
    Long = 4,
    Double = 5,
    String = 6,
    Array = 7,
    Object = 8,
    Resource = 9,
    Reference = 10,
    ConstantExpression = 11,

    Void = 14,
}
