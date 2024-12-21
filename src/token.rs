#[derive(Debug, PartialEq)]
pub struct Token {
    pub(crate) line: usize,
    pub(crate) position: usize,
    pub(crate) kind: Kind,
    pub(crate) value: String
}

#[derive(Debug, PartialEq)]
pub enum Kind {
    // Brackets and delimiters
    RBracket,    // ]
    LBracket,    // [
    RParen,      // )
    LParen,      // (
    LBrace,      // {
    RBrace,      // }
    Semicolon,   // ;
    Comma,       // ,
    
    // Operators
    Equals,      // =
    BinaryOperator,  // +, -, *, /
    ComparisonOperator, // <, >, <=, >=, =
    
    // Special characters
    Underscore,  // _
    ScriptCall,  // ~ (gosub operator)
    
    // Keywords
    Trigger,     // proc, clientscript, etc
    Command,     // calc, map_members, etc
    Def,        // def_int, def_string, etc
    Return,     // return
    If,         // if
    While,      // while
    
    // Identifiers and literals
    Identifier,  // Regular identifiers
    LocalVar,    // $ prefixed variables
    Number,      // Numeric literals
    
    EOF         // End of file marker
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Boolean,
    String,
    Loc,
    Npc,
    Obj,
    Coord,
    NamedObj,
    PlayerUid,
    NpcUid,
    Stat,
    Component,
    Interface,
    Inv,
    Enum,
    Struct,
    Param,
    DbTable,
    DbRow,
    DbColumn,
    Varp,
    MesAnim,
    Category,     // For categorizing items/npcs
    Model,        // 3D model reference
    Animation,    // Animation sequence
    Sound,        // Sound effect
    Color,        // RGB color
    Coord2,       // 2D coordinate (x,y)
    Coord3,       // 3D coordinate (x,y,z)
    IdKit,        // Identity kit reference
    Spotanim,     // Special animation reference
    Varbit,       // Variable bit reference
    Timer,        // Timer reference
}

#[derive(Debug, PartialEq, Clone)]
pub enum TriggerType {
    Proc,          // [proc,name]
    Label,         // [label,name]
    ClientScript,  // [clientscript,name]
    DebugProc,    // [debugproc,name]
    OpLoc,        // [oploc1,name] through [oploc5,name]
    OpNpc,        // [opnpc1,name] through [opnpc5,name]
    OpObj,        // [opobj1,name] through [opobj5,name]
    OpHeld,       // [opheld1,name] through [opheld5,name]
    Button,       // [button,name]
    Timer,        // [timer,name]
    Login,        // [login,name]
    Logout,       // [logout,name]
    IFOpen,       // [if_open,name]
    IFClose,      // [if_close,name]
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    // Math
    Calc,
    Random,
    RandomInc,
    Interpolate,
    
    // Player
    Anim,
    Coord,
    FaceSquare,
    
    // NPC
    NpcAdd,
    NpcDel,
    NpcAnim,
    
    // Objects
    ObjAdd, 
    ObjDel,
    ObjCount,
    
    // Locations
    LocAdd,
    LocDel,
    LocAnim,
    
    // Interface
    IfOpen,
    IfClose,
    IfButton,
    
    // Variables
    SetBit,
    TestBit,
    ToggleBit,
    
    // Database
    DbFind,
    DbFindNext,
    DbGetField,
}