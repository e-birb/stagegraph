use ordered_hash_map::OrderedHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


/// A physical positioning stage for structured light scanning.
///
/// Describes the shape and kinematics of a stage (turntable, linear
/// positioner, tilt unit, etc.) as a **tree of links connected by joints**.
///
/// ```text
/// ┌──────────┐    joint      ┌──────────┐
/// │  link A  │────parent────▶│  link B  │
/// │  (root)  │    child      │          │
/// └──────────┘               └──────────┘
/// ```
///
/// # Semantics
///
/// - Links and joints form a **directed tree** rooted at the first link
///   that is never listed as any joint's `child`.
/// - Each joint's `origin` is relative to its **parent link's frame**.
/// - The `child` link's frame is placed at `joint.origin` and oriented
///   according to the joint type (e.g. rotated around `axis` for revolute).
///
/// # Examples
///
/// A single-axis turntable has one revolute joint:
///
/// ```yaml
/// name: turntable
/// mount: base
/// work: platter
/// links:
///   base:    { }
///   platter: { }
/// joints:
///   rotation:
///     type: revolute
///     parent: base
///     child: platter
///     axis: [0, 0, 1]
///     origin: [0, 0, 0.03]
///     limits: { min: -3.14, max: 3.14 }
/// ```
///
/// More complex stages (tilt + rotate, linear + rotary,
/// Stewart platforms, …) add more joints to the tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StageV0 {
    /// Machine-readable identifier (e.g. `turntable`, `xy-stage`).
    pub name: String,
    /// Free-form description of the stage.
    pub description: Option<String>,

    /// Link identifier of the mounting surface — the part that is fixed
    /// to the world (table, breadboard, robot flange).  This link does
    /// not move when joints are actuated.
    pub mount: String,

    /// Link identifier of the working surface — the part that holds
    /// the object being scanned.  This is the "exit point" of the
    /// kinematic chain: the frame where the scanned object's
    /// coordinate system attaches.
    pub work: String,

    /// The rigid bodies in the stage, keyed by identifier.
    ///
    /// Links referenced by `Joint.parent` / `Joint.child` must exist.
    /// The first link that is never a joint's `child` is the root.
    pub links: OrderedHashMap<String, Link>,

    /// The joints (degrees of freedom) connecting links, keyed by identifier.
    ///
    /// Joints must form a valid tree — no cycles, every `child` references
    /// an existing link, and every non-root link has exactly one parent joint.
    pub joints: OrderedHashMap<String, Joint>,
}

/// A rigid body in the stage.
///
/// Each link can carry a visual geometry for 3D rendering.  Its frame
/// is determined by the parent joint's `origin` and the joint's degree
/// of freedom — not by an explicit position field.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
pub struct Link {
    /// Human-readable display label (optional; defaults to the map key).
    pub label: Option<String>,
    /// Free-form description of this link's role.
    pub description: Option<String>,
    /// Visual geometry attached to this link's frame.
    #[serde(default)]
    pub geometries: Vec<Geometry>,
}

/// A primitive 3D shape attached to a link's frame.
///
/// The shape's `position` and `orientation` are relative to the link's
/// coordinate frame.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Geometry {
    Cylinder(Cylinder),
    Box(BoxShape),
    Mesh(Mesh),
}

/// A cylinder primitive.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
pub struct Cylinder {
    /// Offset from the link frame origin.
    pub position: Option<[f64; 3]>,
    /// Euler angles (radians, XYZ intrinsic) rotating the shape.
    pub orientation: Option<[f64; 3]>,
    pub radius: f64,
    pub height: f64,
}

/// A rectangular box (cuboid) primitive.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
pub struct BoxShape {
    /// Offset from the link frame origin.
    pub position: Option<[f64; 3]>,
    /// Euler angles (radians, XYZ intrinsic) rotating the shape.
    pub orientation: Option<[f64; 3]>,
    /// Extent along the X axis.
    pub width: f64,
    /// Extent along the Y axis.
    pub depth: f64,
    /// Extent along the Z axis.
    pub height: f64,
}

/// A mesh loaded from an external file.
///
/// The file path is relative to the stage description file.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
pub struct Mesh {
    /// Offset from the link frame origin.
    pub position: Option<[f64; 3]>,
    /// Euler angles (radians, XYZ intrinsic) rotating the shape.
    pub orientation: Option<[f64; 3]>,
    /// Path to the mesh file (relative to the stage YAML).
    pub path: String,
    /// Mesh format hint.
    ///
    /// Inferred from the file extension when absent.
    pub format: Option<MeshFormat>,
    /// Uniform scale factor applied to the mesh.
    #[serde(default = "one")]
    pub scale: f64,
}

fn one() -> f64 { 1.0 }

/// Supported mesh formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeshFormat {
    Obj,
    Stl,
    StlBinary,
    Gltf,
    Glb,
}

/// A joint connects two links and contributes one degree of freedom.
///
/// Each joint has a `parent` link and a `child` link, forming a directed
/// edge in the kinematic tree.  The joint's `origin` is relative to the
/// parent link's frame; the child link's frame is computed from it.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
pub struct Joint {
    /// Identifier of the parent link.
    pub parent: String,
    /// Identifier of the child link.
    pub child: String,
    /// The type-specific joint properties.
    #[serde(flatten)]
    pub variant: JointVariant,
}

/// The type-specific properties of a joint.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JointVariant {
    Revolute(RevoluteJoint),
    Prismatic(PrismaticJoint),
}

/// A revolute joint allows rotation around a single axis.
///
/// The `origin` places the child link's frame; the child rotates
/// around `axis` (passing through `origin`) as the joint moves.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
pub struct RevoluteJoint {
    /// Rotation axis in the parent link's frame (does not need
    /// to be unit-length; direction is what matters).
    pub axis: [f64; 3],
    /// Position of the joint in the parent link's frame.
    pub origin: [f64; 3],
    /// Angular limits in radians.  `None` = unlimited rotation.
    pub limits: Option<JointLimits>,
}

/// A prismatic joint allows linear translation along a single axis.
///
/// The `origin` places the child link's frame; the child translates
/// along `axis` as the joint moves.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
pub struct PrismaticJoint {
    /// Translation axis in the parent link's frame.
    pub axis: [f64; 3],
    /// Position of the joint in the parent link's frame.
    pub origin: [f64; 3],
    /// Linear limits in metres.  `None` = unlimited travel.
    pub limits: Option<JointLimits>,
}

/// Motion limits of a joint.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, JsonSchema)]
pub struct JointLimits {
    /// Minimum value (radians for revolute, metres for prismatic).
    pub min: f64,
    /// Maximum value (radians for revolute, metres for prismatic).
    pub max: f64,
}
