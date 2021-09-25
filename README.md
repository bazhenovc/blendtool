# blendtool
Command line tool to extract various data from Blender .blend files.

Currently supports dumping Eevee irradiance volumes to .dds, new features will be added as needed/requested.

## Documentation
```
USAGE:
    blendtool.exe [OPTIONS] --input <input-file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --dump-blend-structure <dump-blend-structure>          Text file to dump .blend file structure
        --dump-irradiance-volumes <dump-irradiance-volumes>    Folder where to dump irradiance volume textures
    -i, --input <input-file>                                   Input .blend file path
```
