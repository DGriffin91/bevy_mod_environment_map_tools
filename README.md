
Currently only encodes Rgba16Float images as rgb9e5 in ktx2 files

More features planned:
- Equirectangular HDR/EXR file input
- Filtering options
- Preview

```
Encode Rgba16Float images as rgb9e5 in ktx2 files

Usage: bevy_mod_environment_map_tools [OPTIONS]

Options:
  -i, --inputs <INPUTS>    Input file paths
  -o, --outputs <OUTPUTS>  Output file paths
  -h, --help               Print help
  -V, --version            Print version
```

Example:
```
cargo run -- --inputs pizzo_pernice_specular.ktx2,pizzo_pernice_diffuse.ktx2 --outputs pizzo_pernice_specular_rgb5e9.ktx2,pizzo_pernice_diffuse_rgb9e5.ktx2
```
