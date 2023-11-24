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
//!         .add_plugins(ExternEventsPlugin::<MyEvent>::default())
//!         .add_systems(Update, event_system);
//!
//!     queue_event(MyEvent::default());
//!
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

        assert_eq!(app.world.resource::<MyEventResource>().0, 1);
    }
}
