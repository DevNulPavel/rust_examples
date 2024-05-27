
struct TubeParams{
    // Tube model
    g0: f32,
    gInf: f32,
    D: f32,
    h: f32,
    // Circuit parameters
    Vb: f32,  // Напряжение базы
    Ra: f32,  // Модель имеет мельчайшие ошибки посравнению с реальной, резистор анодного напряжения
    Vk: f32,  // Напряжение катода для установки смещения
    Rk: f32,  // Rk и Ck формируют фильтр низкой частоты, устанавливает частоту среза 2Гц (form low pass and will be set to a cutoff freq of 2Hz. Usualy you set Vk by choosing proper
    Ck: f32,  // values for Rk and Ck. To make things easier we turn things around an calculate Rk/Ck based on Vk.
              // See adjust_cathode_lp().
}

/// Tube models
// ECC82
const TUBE_CONFIG: TubeParams = TubeParams{
    g0: 9.888576e-15,
    gInf: 6.415385e-24,
    D: 2.95290,
    h: 1.021711e-02,
    Vb: 250.0,
    Ra: 10e3,
    Vk: 6.0,
    Rk: 861.652929,
    Ck: 9.235444e-05
};
// ECC83
// TubeParams{
//     g0: 1.609683e-15,
//     gInf: 2.140844e-23,
//     D: 0.61750,
//     h: 1.000794e-02,
//     Vb: 250.0,
//     Ra: 50e3,
//     Vk: 1.5,
//     Rk: 1430.037044,
//     Ck: 5.564714e-05
// };

struct Tube{
    // constexpr static int m_max_iterations = 6;

    m_first_sample: bool, // true
    m_auto_adjust_cathode_lp: bool, // true
    m_pre_stage: bool, // false
    m_follower_stage: bool, // false
  
    // Tube parameters
    m_g0: f32,
    m_gInf: f32,
    m_D: f32,
    m_h: f32,
    m_h2: f32,
    m_h3: f32,
  
    // Circuit parameters
    m_Ri: f32,      // 100e3;
    m_Rg: f32,      // 1000e3;
    m_Ci: f32,      // 10e-9;
    m_Co: f32,      // 10e-9;
    m_Vgamma: f32,  // 0.6;
  
    // Circuit parameters loaded from the TubeMap
    m_Rk: f32, // 0
    m_Ra: f32, // 0
    m_Ck: f32, // 0
    m_Vk: f32, // 0
    m_Vb: f32, // 0
  
    // Runtime state
    m_Vk_n: f32, // 0
    m_Vk_n1: f32, // 0
    m_Vg_n1: f32, // 0
    m_Va_n1: f32, // 0
    m_Vo_n1: f32, // 0
    m_Vi_n1: f32, // 0
    m_Vgk_n1: f32, // 0
    m_Vak_n1: f32, // 0
    m_Ig_n1: f32, // 0
    m_Ia_n1: f32, // 0
}

impl Tube{
    fn new(params: TubeParams) -> Self {
        Tube{
            m_first_sample:true,
            m_auto_adjust_cathode_lp:true,
            m_pre_stage:false,
            m_follower_stage:false,
          
            // Tube parameters
            m_g0: params.g0,
            m_gInf: params.gInf,
            m_D: params.D,
            m_h: params.h,
            m_h2: params.h.powf(2),
            m_h3: params.h.powf(3),
          
            // Circuit parameters
            m_Ri: 100e3,
            m_Rg: 1000e3,
            m_Ci: 10e-9,
            m_Co: 10e-9,
            m_Vgamma: 0.6,
          
            // Circuit parameters loaded from the TubeMap
            m_Rk: 0.0,
            m_Ra: 0.0,
            m_Ck: 0.0,
            m_Vk: 0.0,
            m_Vb: 0.0,

            // Runtime state
            m_Vk_n: 0.0,
            m_Vk_n1: 0.0,
            m_Vg_n1: 0.0,
            m_Va_n1: 0.0,
            m_Vo_n1: 0.0,
            m_Vi_n1: 0.0,
            m_Vgk_n1: 0.0,
            m_Vak_n1: 0.0,
            m_Ig_n1: 0.0,
            m_Ia_n1: 0.0,
        }
    }

    fn init_state(&mut self) {
        self.m_Vg_n1 = 0.0; 
        self.m_Vo_n1 = 0.0;
        self.m_Vi_n1 = 0.0;
        self.m_Ig_n1 = 0.0;
        self.m_Ia_n1 = 0.0;
        self.m_Va_n1 = self.m_Vb;
        self.m_Vak_n1 = self.m_Vb;

        self.m_Vk_n = self.m_Vk;
        self.m_Vk_n1 = self.m_Vk;

        self.m_Vgk_n1 = self.m_Vg_n1 - self.m_Vk_n1;
        
        self.input_delay = self.output_delay = 1.0;
      }
}