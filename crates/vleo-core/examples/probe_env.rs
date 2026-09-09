fn main() {
    use vleo_core::physics::env::*;
    use vleo_units::*;
    for &(f107, f107a, kp, name) in &[(70.0,70.0,1.0,"solar min quiet"),(150.0,150.0,3.0,"moderate"),(250.0,250.0,7.0,"solar max storm")] {
        let t_inf = exospheric_temperature(f107, f107a, kp);
        println!("--- {name}: T_inf = {:.1} K", t_inf.get());
        for km in [120.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0] {
            let h = Length::from_km(km);
            let c = composition(h, t_inf);
            println!("  {km:5.0} km  rho={:9.3e} kg/m3  n={:9.3e}  M={:5.2} g/mol  O={:4.1}%  H={:5.1} km  T={:6.1} K",
                c.mass_density().get(), c.total().get(), c.mean_molar_mass().get()*1000.0,
                c.atomic_oxygen_fraction().get()*100.0, scale_height(h,t_inf).km(), temperature(h,t_inf).get());
        }
    }
}
