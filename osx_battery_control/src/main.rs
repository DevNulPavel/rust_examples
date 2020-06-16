include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main(){
    unsafe{
        smc_init();
        smc_close();
    }
}