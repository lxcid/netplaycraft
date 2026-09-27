use netplaycraft_counter::run_demo;
use netplaycraft_sim::Checksummed;
use netplaycraft_transport_memory::NetworkConditions;

fn main() {
    let result = run_demo(
        NetworkConditions {
            latency_ms: 80,
            jitter_ms: 30,
            loss_bps: 300,
            duplicate_bps: 200,
            reorder_bps: 200,
            reorder_delay_ms: 60,
        },
        42,
    );
    match result {
        Ok(report) => {
            println!(
                "host:    count={} next_tick={} turns={} checksum={:016x}",
                report.host.count(),
                report.host.next_tick().0,
                report.host.turn().0,
                report.host.checksum().0
            );
            println!(
                "replica: count={} next_tick={} turns={} checksum={:016x}",
                report.replica.count(),
                report.replica.next_tick().0,
                report.replica.turn().0,
                report.replica.checksum().0
            );
            println!(
                "network: {:?}; virtual steps={}",
                report.stats, report.network_steps
            );
        }
        Err(error) => {
            eprintln!("counter failed: {error:?}");
            std::process::exit(1);
        }
    }
}
