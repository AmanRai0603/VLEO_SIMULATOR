//! End-to-end sanity probe: the closure thread, by hand.
fn main() {
    use vleo_core::physics::*;
    use vleo_units::*;

    let f107 = 150.0; let f107a = 150.0; let kp = 3.0;
    let t_inf = env::exospheric_temperature(f107, f107a, kp);
    println!("T_inf {:.0} K", t_inf.get());
    println!("{:>6} {:>10} {:>7} {:>7} {:>9} {:>9} {:>8} {:>8} {:>7}",
        "alt", "rho", "V", "Cd", "drag", "mdot", "thrust", "P_bus", "T/D");
    for km in [180.0, 200.0, 225.0, 250.0, 275.0, 300.0, 350.0] {
        let h = Length::from_km(km);
        let r = orbit::radius(h);
        let v = orbit::circular_velocity(r);
        let comp = env::composition(h, t_inf);
        let rho = comp.mass_density();
        let m_mean = comp.mean_molar_mass();
        let t_local = env::temperature(h, t_inf);
        let s = aero::speed_ratio(v, t_local, m_mean).get();
        let alpha = aero::accommodation_coefficient(comp.o, t_inf);
        let t_wall = Temperature::new(300.0);
        let cd = aero::cylinder_drag_coefficient(s, Length::new(2.0), Length::new(0.6), alpha, t_wall, v, m_mean);
        let a_ref = Area::new(0.283);          // 0.6 m diameter frontal
        let a_in  = Area::new(0.2);            // intake mouth
        let drag = aero::drag_force(rho, v, cd, Area::new(a_ref.get() + a_in.get()));
        let intake = prop::intake_balance(comp.total(), v, a_in, Area::new(0.01),
            Ratio::new(0.90), Ratio::new(0.06), Temperature::new(600.0), m_mean);
        let mdot = prop::collected_mass_flow(rho, v, a_in, intake.collection_efficiency);
        let mdot_i = prop::ion_mass_flow(mdot, Ratio::new(0.40));
        let ve = prop::beam_exhaust_velocity(Voltage::new(1200.0), m_mean);
        let thrust = prop::beam_thrust(mdot_i, ve, Ratio::new(0.97), Ratio::new(0.97));
        let pj = prop::jet_power(thrust, mdot_i);
        let pi = prop::ionisation_power(mdot_i, m_mean, 250.0);
        let pin = prop::thruster_input_power(pj, pi, Ratio::new(0.15));
        let pbus = prop::bus_power_demand(pin, Ratio::new(0.90));
        let td = prop::thrust_to_drag(thrust, drag);
        println!("{km:6.0} {:10.3e} {:7.0} {:7.3} {:8.3}mN {:8.3}mg/s {:7.3}mN {:7.0}W {:7.3}",
            rho.get(), v.get(), cd, drag.mn(), mdot.mg_s(), thrust.mn(), pbus.get(), td.get());
        if km == 250.0 {
            println!("      -> eta_c {:.3}  CR {:.1}  alpha {:.3}  s {:.2}  Isp {:.0}s  eta_tot {:.4}",
                intake.collection_efficiency.get(), intake.compression_ratio.get(), alpha, s,
                prop::specific_impulse(thrust, mdot).get(), prop::total_efficiency(pj, pbus).get());
        }
    }
}
