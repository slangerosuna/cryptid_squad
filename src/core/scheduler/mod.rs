use crate::core::*;
use tokio::time::Instant;

pub struct Scheduler {
    pub init_systems: Vec<System>,
    pub update_systems: Vec<System>,
    pub fixed_update_systems: Vec<System>,

    init_execution_order: Vec<Vec<usize>>,
    update_execution_order: Vec<Vec<usize>>,
    fixed_update_execution_order: Vec<Vec<usize>>,

    pub fixed_update_interval: f64,
    pub start_time: Instant,
    prev_time: f64,
}

impl Scheduler {
    pub fn new(fixed_update_interval: f64) -> Scheduler {
        Scheduler {
            init_systems: Vec::new(),
            update_systems: Vec::new(),
            fixed_update_systems: Vec::new(),
            init_execution_order: Vec::new(),
            update_execution_order: Vec::new(),
            fixed_update_execution_order: Vec::new(),

            fixed_update_interval,
            start_time: Instant::now(),
            prev_time: 0.0,
        }
    }

    pub fn add_system(&mut self, system: System, system_type: SystemType) {
        match system_type {
            SystemType::Init => self.init_systems.push(system),
            SystemType::Update => self.update_systems.push(system),
            SystemType::FixedUpdate => self.fixed_update_systems.push(system),
        };

        self.generate_execution_order();
    }

    pub async fn update(&mut self, game_state: &mut GameState) {
        let time = self.get_time();
        let dt = time - self.prev_time;
        self.prev_time = time;

        for group in self.update_execution_order.iter() {
            let mut futures = Vec::with_capacity(group.len());

            // Run all systems in the group
            for system_index in group.iter() {
                let system = &self.update_systems[*system_index];

                // create a new reference to the game state so you can pass it to the async block
                // and have multiple mutable references to the game state
                // this is why we must ensure that we accurately track the system's args to avoid
                // race conditions
                let game_state = unsafe { &mut *(game_state as *mut GameState) };

                futures.push((system.system)(game_state, time, dt));
            }
            // Wait for all futures to complete
            for future in futures {
                future.await;
            }
        }
    }

    pub fn generate_execution_order(&mut self) {
        self.init_execution_order = self.generate_execution_order_for_systems(&self.init_systems);
        self.update_execution_order = self.generate_execution_order_for_systems(&self.update_systems);
        self.fixed_update_execution_order = self.generate_execution_order_for_systems(&self.fixed_update_systems);
    }

    fn generate_execution_order_for_systems(&self, systems: &Vec<System>) -> Vec<Vec<usize>> {
        let mut execution_order = Vec::new();
        let mut visited = vec![false; systems.len()];

        for i in 0..systems.len() {
            if !visited[i] {
                let mut group = vec![i];
                let mut dissallowed_components = systems[i].args.clone();

                for j in 0..systems.len() {
                    if !visited[j] {
                        let mut can_run = true;
                        for component in &systems[j].args {
                            if dissallowed_components.contains(component) {
                                can_run = false;
                                break;
                            }
                        }
                        if can_run {
                            group.push(j);
                            visited[j] = true;
                            for component in &systems[j].args {
                                dissallowed_components.push(*component);
                            }
                        }
                    }
                }

                execution_order.push(group);
            }
        }

        execution_order
    }

    pub fn get_time(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}
