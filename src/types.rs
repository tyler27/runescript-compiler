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