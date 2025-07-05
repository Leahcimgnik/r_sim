use std::collections::{VecDeque, HashMap};


fn main() {

    // let people:Vec<Person> = vec![
    //     Person{id:1,spawn_time:1,process_time:5},
    //     Person{id:2,spawn_time:2,process_time:5},
    //     Person{id:3,spawn_time:3,process_time:5},
    //     Person{id:4,spawn_time:4,process_time:5},
    //     Person{id:5,spawn_time:5,process_time:5},
    //     Person{id:6,spawn_time:6,process_time:5},
    // ];

    let person_a:Person = Person {id:1,spawn_time:10,process_time:1};
    let person_b:Person = Person {id:2,spawn_time:5,process_time:7};
    let mut sim_env:Environment = Environment::new();

    person_a.process(&mut sim_env);
    person_b.process(&mut sim_env);

    println!("{:?}",sim_env.event_list);

    sim_env.run_sim();


    // let mut servers:Vec<Server> = Vec::new();
    // let num_servers:u8 = 2;

    // for id in 0..num_servers {
    //     servers.push(Server::new(id+1));
    // }

    // sim(people,servers);
    // // random_sampling_data();

}

struct Environment {
    event_list:HashMap<u64,VecDeque<(u64,String,u64)>>,
}

impl Environment {

    fn new() -> Environment {
        Environment { event_list:HashMap::new() }
    }

    fn timeout(&mut self, person:&Person, time_consumption:u64) {

        self.event_list
            .entry(person.id)
            .or_insert_with(VecDeque::new)
            .push_back((person.id,"timeout".to_string(),time_consumption));

    }

    fn get_resource(&mut self, person:&Person, process_time:u64) {

        self.event_list
            .entry(person.id)
            .or_insert_with(VecDeque::new)
            .push_back((person.id,"get_resource".to_string(),process_time));

    }

    // fn log(&mut self, person:&Person, message:String) {
    //     self.event_list.push_back((person.id,"log".to_string(),0));
    //     println!("{:?}, {}", person, message);
    // }


    fn _organise_initial_events(&mut self) -> VecDeque<(u64,String,u64,u64)> {
        /*
        The first event for each agent is moved to a scheduled events list.
        Any agents that are left with an empty event list are removed from the HashMap.
         */

        let mut scheduled_events:VecDeque<(u64,String,u64,u64)> = VecDeque::new();
        let mut keys_to_remove:Vec<u64> = Vec::new();

        for (key, queue) in self.event_list.iter_mut() {
            if let Some(event) = queue.pop_front() {
                scheduled_events.push_back((event.0,event.1,event.2,0));

                if queue.is_empty() {
                    keys_to_remove.push(*key);
                }
            }
        }

        for key in keys_to_remove {
            self.event_list.remove(&key);
        }

        return scheduled_events

    }

    fn run_sim(&mut self) {
        
        let mut sim_time:u64 = 0;

        // id, event_type, sim_time, scheduled time.
        let mut scheduled_events:VecDeque<(u64,String,u64,u64)> = self._organise_initial_events();
        let mut count:u16 = u16::MAX;


        while (scheduled_events.len() > 0) & (count > 0) {

            scheduled_events.retain(|event| {
                if event.3 == sim_time {
                    match event.1.as_str() {
                        "timeout" => {
                            

                        }

                        "get_resource" => {

                        }

                        _ => {
                            println!("Unknown event type: {:?}", event.1);
                        }
                    }
                    false
                } else {
                    true
                }

            });



            count -= 1;
        }
        
        println!("{:?}", scheduled_events);

        println!("{:?}", self.event_list);


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
        // sim_env.log(&self, "I've spawned".to_string());
        sim_env.get_resource(&self, self.process_time);
        // sim_env.log(&self, "Finished with resource".to_string());
    }

}
