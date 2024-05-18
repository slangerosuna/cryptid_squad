use crate::*;
use steamworks::*;

pub struct NetworkingResource {
    pub client: Client,
    pub single: SingleClient,
}
impl_resource!(NetworkingResource, 7);

impl NetworkingResource {
    pub fn new() -> Result<Self, SteamError> {
        let (client, single) = Client::init_app(480)?;

        let _cb = client.register_callback(|p: PersonaStateChange| {
            println!("Got callback: {:?}", p);
        });

        let utils = client.utils();
        println!("Utils:");
        println!("AppId: {:?}", utils.app_id());
        println!("UI Language: {}", utils.ui_language());

        let apps = client.apps();
        println!("Apps");
        println!("IsInstalled(480): {}", apps.is_app_installed(AppId(480)));
        println!("InstallDir(480): {}", apps.app_install_dir(AppId(480)));
        println!("BuildId: {}", apps.app_build_id());
        println!("AppOwner: {:?}", apps.app_owner());
        println!("Langs: {:?}", apps.available_game_languages());
        println!("Lang: {}", apps.current_game_language());
        println!("Beta: {:?}", apps.current_beta_name());

        let friends = client.friends();
        println!("Friends");
        let list = friends.get_friends(FriendFlags::IMMEDIATE);
        for f in &list {
            println!("Friend: {:?} - {}({:?})", f.id(), f.name(), f.state());
            friends.request_user_information(f.id(), true);
        }

        Ok(Self {
            client,
            single,
        })
    }
}

const FRAMES_PER_CALLBACK_RUN: u32 = 3;
static mut FRAMES_SINCE_CALLBACK_RUN: u32 = 0;

create_system!(run_callbacks, get_run_callbacks_system;
    uses NetworkingResource);
async fn run_callbacks(game_state: &mut GameState, _t: f64, _dt: f64) {
    unsafe {
        // the system is never run in parallel with itself, so `FRAMES_SINCE_CALLBACK_RUN` doesn't
        // need to be atomized
        FRAMES_SINCE_CALLBACK_RUN += 1;
        if FRAMES_SINCE_CALLBACK_RUN < FRAMES_PER_CALLBACK_RUN
          { return; }

        FRAMES_SINCE_CALLBACK_RUN = 0;
    }
    let networking = game_state.get_resource::<NetworkingResource>().unwrap();
    networking.single.run_callbacks();
}

create_system!(init_networking, get_init_networking_system;
    uses GameState, NetworkingResource);
async fn init_networking(game_state: &mut GameState, _t: f64, _dt: f64) {
    match NetworkingResource::new() {
        Ok(networking) => {
            let scheduler = game_state.get_scheduler_mut();
            //add systems here
            scheduler.add_system(get_run_callbacks_system(), SystemType::Update);

            game_state.add_resource(networking);
        },
        Err(e) => {
            eprintln!("Error initializing networking: {}", e);
            if game_state.conf.exit_on_networking_error {
                game_state.close();
            }
        }
    }
}
