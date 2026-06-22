# stagegraph

A YAML format for describing the kinematics and geometry of positioning
stages, primarily used with [e-light](https://e-birb.com/projects/e-light/).

Inspired by URDF, but simplified for scanning workflows.

## The idea

A stage is a **tree of rigid links connected by joints**. Each joint
contributes one degree of freedom (rotation or translation). This lets
you describe simple turntables as well as multi-axis positioners.

## Resources

- [Schema](doc/stagegraph.schema.json)
- [Example](examples/robot.yaml)
