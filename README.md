# bevy_simpletoon

[![Latest version](https://img.shields.io/crates/v/bevy_simpletoon.svg)](https://crates.io/crates/bevy_simpletoon)

## WARNING

This is still in development and is VERY early days.
I've decided to use it myself to make a game and will be updating it as I build out the game.

## Usage

```rust

    fn main() {
        // add the SimpletoonPlugin to your app.
        // note that DefaultPlugin is not required.
        App::new()
            .add_plugins((
                DefaultPlugins,
                SimpletoonPlugin,
        )).run();
    }
    // add SimpletoonSettings to your camera
    // note that Msaa *must* be Off, as it is not currently compatible with post processing
    fn setup_camera(mut commands: Commands) {
        commands.spawn((
            Camera3d::default(),
            SimpletoonSettings::default(),  // These settings can be updated any time to change the shader's effect
            Msaa::Off,
            // Depth and normal prepass and both currently required by the shader.
            DepthPrepass,
            NormalPrepass,
            //Fxaa::default()   // Fxaa is not required, though the shader comes with no built-in anti-aliasing.
        ));
    }

```

## Compatibility

| Bevy version | `bevy_simpletoon` version |
| :----------- | :------------------------ |
| `0.16`       | `0.2`                     |
| `0.15`       | `0.1`                     |

## Tips

> For a more detailed example, check the examples directory.

> Though I've tweaked the default settings to work for me, it does not mean they will work well for your scene, so you may need to play around with them to see what fits.

> This shader works best with high roughness materials and simple or no textures, as demonstrated in examples/shapes.rs
