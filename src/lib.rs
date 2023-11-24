use std::{marker::PhantomData, sync::Mutex};

use bevy::prelude::*;
use generic_global_variables::*;
use once_cell::sync::OnceCell;

#[derive(Event, Default)]
pub struct ExternEvent<T: Send + Sync + Default>(pub T);

#[derive(Resource)]
pub struct ExternEventQueueResource<T: Send + Sync + Default>(T);

#[derive(Default)]
pub struct ExternEventsPlugin<T>(PhantomData<T>);

impl<T: Send + Sync + Default> Plugin for ExternEventsPlugin<T>
where
    ExternEvent<T>: Event,
{
    fn build(&self, app: &mut App) {
        app.add_event::<ExternEvent<T>>()
            .add_systems(PreUpdate, poll_events);
    }
}

fn get_global<T: Send + Sync>(f: impl FnOnce() -> T) -> Entry<T> {
    static GLOBALS: OnceCell<GenericGlobal> = OnceCell::new();

    let globals = GLOBALS.get_or_init(GenericGlobal::new);
    globals.get_or_init(f)
}

pub fn queue_event<T: 'static + Send + Sync>(event: T) {
    let arc = get_global(Mutex::<Vec<T>>::default);
    arc.lock().unwrap().push(event);
}

fn poll_events<T: 'static + Send + Sync + Default>(mut writer: EventWriter<ExternEvent<T>>) {
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
