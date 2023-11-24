//! # Why
//!
//! Because at some point you might want to interact with code outside of Bevy
//! (External SDKs, Native Platform Code, non-Bevy crates).
//! With the help of this crate you can queue events from anywhere and
//! they will be available via the typical `EventReader` mechanism inside your Bevy Systems.
//!
//! **Note** that this comes at the cost of us having a global static `RwLock`-based Queue
//! that we poll every frame (`PreUpdate`) to forward into an `EventWriter`.
//! Events are Boxed because I found no other way of having a global static generic Datatype without using `Any`.
//!
//! Therefore I suggest using this for non-every-frame interaction and rolling a custom solution otherwise.
//!
//! # Example:
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_extern_events::{queue_event, ExternEvent, ExternEventsPlugin};
//!
//! #[derive(Default)]
//! pub struct MyEvent;
//!
//! #[derive(Resource, Reflect, Default)]
//! pub struct MyEventResource(i32);
//!
//! pub fn event_system(
//!     mut res: ResMut<MyEventResource>,
//!     mut native_events: EventReader<ExternEvent<MyEvent>>,
//! ) {
//!     for _e in native_events.read() {
//!         res.0 += 1;
//!     }
//! }
//!
//! fn test() {
//!     let mut app = App::new();
//!     app.init_resource::<MyEventResource>()
//!         // register `ExternEventsPlugin` with our event type
//!         .add_plugins(ExternEventsPlugin::<MyEvent>::default())
//!         // register our system that will react to these events
//!         .add_systems(Update, event_system);
//!     
//!     // can be called any thread, from anywhere (for example c ffi)
//!     queue_event(MyEvent::default());
//!
//!     // next pre-update will forward extern events to the bevy events system
//!     // this will trigger `event_system` of this example
//!     app.update();
//!
//!     assert_eq!(app.world.resource::<MyEventResource>().0, 1);
//! }
//! ```
use std::{marker::PhantomData, sync::Mutex};

use bevy::prelude::*;
use generic_global_variables::*;
use once_cell::sync::OnceCell;

/// wrapper from external events
#[derive(Event, Default)]
pub struct ExternEvent<T: Send + Sync + Default>(pub T);

/// Bevy plugin for convenient proper installation.
/// Registers the event and the polling systems.
#[derive(Default)]
pub struct ExternEventsPlugin<T>(PhantomData<T>);

impl<T: Send + Sync + Default> Plugin for ExternEventsPlugin<T>
where
    ExternEvent<T>: Event,
{
    fn build(&self, app: &mut App) {
        app.add_event::<ExternEvent<T>>()
            .add_systems(PreUpdate, poll_events_system);
    }
}

/// external entry point to queue events from anywhere from any thread
pub fn queue_event<T: 'static + Send + Sync>(event: T) {
    let arc = get_global(Mutex::<Vec<T>>::default);
    arc.lock().unwrap().push(event);
}

/// solutionn to do generic global statics using `generic_global_variables`
fn get_global<T: Send + Sync>(f: impl FnOnce() -> T) -> Entry<T> {
    static GLOBALS: OnceCell<GenericGlobal> = OnceCell::new();

    let globals = GLOBALS.get_or_init(GenericGlobal::new);
    globals.get_or_init(f)
}

/// bevy system to run `PreUpdate` polling the event queue and pushing each event into the `EventWriter`
fn poll_events_system<T: 'static + Send + Sync + Default>(mut writer: EventWriter<ExternEvent<T>>) {
    let arc = get_global(Mutex::<Vec<T>>::default);
    while let Some(e) = arc.lock().unwrap().pop() {
        info!("poll event");
        writer.send(ExternEvent(e));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::{queue_event, ExternEvent, ExternEventsPlugin};

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

    #[test]
    fn smoke() {
        let mut app = App::new();
        app.init_resource::<MyEventResource>()
            .add_plugins(ExternEventsPlugin::<MyEvent>::default())
            .add_systems(Update, event_system);

        queue_event(MyEvent::default());

        app.update();

        assert_eq!(app.world.resource::<MyEventResource>().0, 1);

        app.update();

        // make sure no other event was forwarded
        assert_eq!(app.world.resource::<MyEventResource>().0, 1);
    }
}
