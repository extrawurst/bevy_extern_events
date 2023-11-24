# bevy_extern_events

[![crates.io](https://img.shields.io/crates/v/bevy_extern_events.svg)](https://crates.io/crates/bevy_extern_events)
[![docs](https://docs.rs/bevy_extern_events/badge.svg)](https://docs.rs/bevy_extern_events)

## Why?

Because at some point you might want to interact with code outside of Bevy (External SDKs, Native Platform Code, non-Bevy crates). With the help of this crate you can queue events from anywhere and they will be available via the typical `EventReader` mechanism inside your Bevy Systems.

**Note** that this comes at the cost of us having a global static `RwLock`-based Queue that we poll every frame (`PreUpdate`) to forward into an `EventWriter`. Events are `Box`ed because I found no other way of having a global static generic Datatype without using `Any`. 

Therefore I suggest using this for non-every-frame interaction and rolling a custom solution otherwise.

## Example:

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
