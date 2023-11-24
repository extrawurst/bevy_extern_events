# bevy_extern_events

[![crates.io](https://img.shields.io/crates/v/bevy_extern_events.svg)](https://crates.io/crates/bevy_extern_events)
[![docs](https://docs.rs/bevy_extern_events/badge.svg)](https://docs.rs/bevy_extern_events)

usage example: 

```rust
#[derive(Default)]
pub struct MyEvent;

#[derive(Resource, Reflect, Default)]
pub struct MyEventResource(i32);

pub fn event_system(
    mut res: ResMut<MyEventResource>,
    mut native_events: EventReader<ExternEvent<MyEvent>>,
) {
    for _e in native_events.read() {
        res.0 += 1;
    }
}

fn test() {
    let mut app = App::new();
    app.init_resource::<MyEventResource>()
        .add_plugins(ExternEventsPlugin::<MyEvent>::default())
        .add_systems(Update, event_system);

    queue_event(MyEvent::default());

    app.update();

    assert_eq!(app.world.resource::<MyEventResource>().0, 1);
}
```

## TODO

- [ ] CI
- [ ] clippy