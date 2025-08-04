use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Reverse;
use std::fmt;
use std::thread::spawn;



struct Environment {
    event_list:HashMap<u64,VecDeque<IdleEvent>>,
    resources:HashMap<String,Resource>,
    resource_queues:HashMap<String,VecDeque<(u64,u64)>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            event_list:HashMap::new(),
            resources:HashMap::new(),
            resource_queues:HashMap::new(),        
        }
    }
}

impl Environment {

    fn timeout(&mut self, person:&Person, process_time:u64) {

        self.event_list
            .entry(person.id)
            .or_insert_with(VecDeque::new)
            .push_back(
                IdleEvent {
                    id:person.id,
                    event_type:EventType::Timeout,
                    process_time:process_time,
                    target:String::new(),
                }
            );
    }

    fn enter_queue(&mut self, person:&Person, process_time:u64, target:String) {

        self.event_list
            .entry(person.id)
            .or_insert_with(VecDeque::new)
            .push_back(
                IdleEvent {
                    id:person.id,
                    event_type:EventType::EnterQueue,
                    process_time:process_time,
                    target:target
                }
            );

    }

    fn add_resource(&mut self, resource:Resource) {

        self.resources
            .entry(resource.resource_name.clone())
            .or_insert(resource);

    }

    fn add_queue(&mut self, queue:SimulationQueue) {

        self.resource_queues
            .entry(queue.queue_name)
            .or_insert(queue.queue);

    }

    fn resource_interruption(
        &mut self,
        id:u64,
        spawn_time:u64,
        target:String,
        capacity_interrupted:u64,
        duration:u64,
    ) {

    }

    fn _organise_initial_events(&mut self) -> BinaryHeap<(Reverse<u64>, ActiveEvent)> {
        /*
        The first event for each agent is moved to a scheduled events list.
        Any agents that are left with an empty event list are removed from the HashMap.
         */

        let mut scheduled_events: BinaryHeap<(Reverse<u64>, ActiveEvent)> = BinaryHeap::new();
        let mut keys_to_remove:Vec<u64> = Vec::new();
        let sim_time:u64 = 0;

        for (key, queue) in self.event_list.iter_mut() {
            if let Some(idle_event) = queue.pop_front() {
                scheduled_events.push((Reverse(sim_time), ActiveEvent::from_idle_event(idle_event, sim_time)));

                if queue.is_empty() {
                    keys_to_remove.push(*key);
                }
            }
        }

        for key in keys_to_remove {
            self.event_list.remove(&key);
        }

        scheduled_events

    }

    fn run_sim(&mut self) -> Vec<String> {

        let mut sim_logs:Vec<String> = Vec::new();
        let mut sim_time:u64 = 0;
        let mut loops:u16 = 1000;
        let mut scheduled_events:BinaryHeap<(Reverse<u64>, ActiveEvent)> = self._organise_initial_events();
        let mut staged_events:Vec<ActiveEvent> = Vec::new();


        while loops > 0 {

            let mut keys_to_remove:Vec<u64> = Vec::new();

            scheduled_events.retain(|(_, event)| {
                if event.scheduled_time == sim_time {
                    
                    match event.event_type {

                        EventType::Timeout => {

                            sim_logs.push(
                                format!("{}: Person id {} is executing event {}.", sim_time, event.id, event.event_type)
                            );

                            // sim time + timeout time.
                            staged_events.push(
                                ActiveEvent {
                                    id:event.id,
                                    event_type:EventType::EndTimeout,
                                    process_time:0,
                                    target:event.target.clone(),
                                    scheduled_time:sim_time+event.process_time
                                }
                            );

                        }

                        EventType::EndTimeout => {

                            sim_logs.push(
                                format!("{}: Person id {} is executing event {}.", sim_time, event.id, event.event_type)
                            );

                            // Pop the next event for this id, and schedule it to be current.
                            // sim time + timeout time.
                            if let Some(next_event_for_agent) = self.event_list.get_mut(&event.id) {
                                if let Some(idle_event) = next_event_for_agent.pop_front() {
                                    staged_events.push(ActiveEvent::from_idle_event(idle_event, sim_time));
                                }
                            }

                            if let Some(queue) = self.event_list.get(&event.id) {
                                if queue.is_empty() {
                                    keys_to_remove.push(event.id);
                                }
                            }

                        }

                        EventType::EnterQueue => {

                            sim_logs.push(
                                format!("{}: Person id {} is being added to {}.", sim_time, event.id, event.target)
                            );

                            self.resource_queues
                                .get_mut(&event.target)
                                .expect("User defined queue name.")
                                .push_back((event.id, event.process_time));

                        }

                        EventType::ResourceInterruption => {

                        }

                    }
                    false
                } else {
                    true
                }

            });

            // Check if resource releases an agent at this time.
            for (resource_name, resource) in self.resources.iter_mut() {
                for (resource_capacity_id, resource_capacity) in resource
                    .status
                    .iter_mut()
                    .filter(|(_, resource_capacity)| resource_capacity.available == sim_time)
                {

                    sim_logs.push(
                        format!(
                            "{}: {}, id {}, released Person id {}.",
                            sim_time, resource_name, resource_capacity_id, resource_capacity.current_agent_id
                        )
                    );

                    if let Some(event_queue) = self.event_list.get_mut(&resource_capacity.current_agent_id) {
                        if let Some(idle_event) = event_queue.pop_front() {

                            sim_logs.push(
                                format!(
                                    "{}: Person id {} has event type {} moved from events list to scheduled events.",
                                    sim_time, resource_capacity.current_agent_id, idle_event.event_type
                                )
                            );

                            staged_events.push(ActiveEvent::from_idle_event(idle_event, sim_time));

                        }
                    }

                    resource_capacity.available = u64::MAX;
                    resource_capacity.current_agent_id = u64::MAX;

                }

            }


            // For each resource... If resource is available and resource queue is not empty, then put agent into resource.
            for resource in self.resources.values_mut() {
                if let Some(queue) = self.resource_queues.get_mut(&resource.queue_target) {
                    if queue.is_empty() {
                        continue;
                    }
                    for (resource_capacity_id, resource_capacity) in resource
                        .status
                        .iter_mut()
                        .filter(|(_, cap)| cap.available == u64::MAX)
                    {
                        if let Some((agent_id, agent_sim_time)) = queue.pop_front() {
                            sim_logs.push(
                                format!(
                                    "{}: Adding Person id {} from {} to {} id {}.",
                                    sim_time, agent_id, resource.queue_target, resource.resource_name, resource_capacity_id
                                )
                            );

                            resource_capacity.current_agent_id = agent_id;
                            resource_capacity.available = agent_sim_time + sim_time;

                        } else {
                            break; // queue is now empty.
                        }
                    }
                }
            }

            // Move staged_events into scheduled_events.
            while !staged_events.is_empty() {
                let moving_event = staged_events.pop().unwrap();
                scheduled_events.push((Reverse(moving_event.scheduled_time),moving_event));
            }


            // Remove any agents with no remaining events.
            for key in keys_to_remove {
                self.event_list.remove(&key);
            }

            // Check if sim is finished.
            if scheduled_events.is_empty()
                && self.resources.iter().all(|(_, res)| {
                    res.status.values().all(|status| status.available == u64::MAX)
                })
            {
                sim_logs.push(
                    format!("{}: Simulation finished.", sim_time)
                );

                break;

            }

            // Update the sim clock.
            if let Some(min_resource_available_time) = self.resources
                .values()
                .flat_map(|res| res.status.values().map(|s| s.available))
                .min()
            {
                if let Some((_, min_scheduled_event_time)) = scheduled_events.peek() {
                    sim_time = min_scheduled_event_time.scheduled_time.min(min_resource_available_time);
                } else {
                    sim_time = min_resource_available_time;
                }
            }

            loops -= 1;
        }
        
        sim_logs


    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum EventType {
    Timeout,
    EndTimeout,
    EnterQueue,
    ResourceInterruption,
}

impl fmt::Display for EventType {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EventType::Timeout => "timeout",
            EventType::EndTimeout => "end_timeout",
            EventType::EnterQueue => "enter_queue",
            EventType::ResourceInterruption => "resource_interruption",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
struct Resource {
    resource_name:String,
    queue_target:String,
    status:HashMap<u64, ResourceStatus>,
}

impl Resource {

    fn new_resource(capacity:u64, resource_name:String, queue_target:String) -> Resource {

        let mut new_resource:Resource = Resource {
            resource_name:resource_name,
            queue_target:queue_target,
            status:HashMap::new()
        };

        for cap in 1..capacity+1 {
            new_resource
                .status
                .entry(cap)
                .or_insert(ResourceStatus {
                    available:u64::MAX,
                    current_agent_id:u64::MAX
                });
        }

        new_resource

    }
}

#[derive(Debug)]
struct ResourceStatus {
    available:u64,
    current_agent_id:u64
}

struct SimulationQueue {
    queue_name:String,
    queue:VecDeque<(u64,u64)>,
}

#[derive(Debug)]
struct IdleEvent {
    id:u64,
    event_type:EventType,
    process_time:u64,
    target:String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ActiveEvent {
    id:u64,
    event_type:EventType,
    process_time:u64,
    target:String,
    scheduled_time:u64,
}

impl ActiveEvent {
    fn from_idle_event(idle_event:IdleEvent, sim_time:u64) -> ActiveEvent {
        ActiveEvent {
            id:idle_event.id,
            event_type:idle_event.event_type,
            process_time:idle_event.process_time,
            target:idle_event.target,
            scheduled_time:sim_time,
        }
    }
}

impl Ord for ActiveEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id) // or any stable fallback comparison
    }
}

impl PartialOrd for ActiveEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct Person {
    id:u64,
    spawn_time:u64,
    process_time:u64,
}



impl Person {

    fn process(self, sim_env:&mut Environment) {
        sim_env.timeout(&self, self.spawn_time);
        sim_env.enter_queue(&self, self.process_time, "queue_1".to_string());
    }

}

struct ResourceInterruption {
    id:u64,
}

impl ResourceInterruption {

    fn add_interruption(
        &self,
        spawn_time:u64,
        target:String,
        capacity_interrupted:u64,
        duration:u64,
        sim_env:&mut Environment,
    ) {
        sim_env.resource_interruption(
            self.id,
            spawn_time,
            target,
            capacity_interrupted,
            duration,
        );
    }

}


fn main() {

    let people:Vec<Person> = vec![
        Person {id:1,spawn_time:3,process_time:1},
        Person {id:2,spawn_time:3,process_time:7},
        Person {id:3,spawn_time:3,process_time:10},
    ];

    let mut sim_env:Environment = Environment::default();
    let resource_queue:SimulationQueue = SimulationQueue { queue_name: "queue_1".to_string(), queue: VecDeque::new() };
    let resource_a:Resource = Resource::new_resource(2, "resource_1".to_string(), "queue_1".to_string());
    let resource_b:Resource = Resource::new_resource(1, "resource_2".to_string(), "queue_1".to_string());
    
    for person in people.into_iter() {
        person.process(&mut sim_env);
    }

    sim_env.add_resource(resource_a);
    sim_env.add_resource(resource_b);
    sim_env.add_queue(resource_queue);


    let sim_logs:Vec<String> = sim_env.run_sim();

    for log in sim_logs {
        println!("{}", log);
    }

    /*
    TODO:
    - Add resource interruptions
    - Add agent interruptions
    - Add parallelism
    */


}

