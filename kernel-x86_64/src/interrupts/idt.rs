use crate::interrupts::{InterruptVector, exceptions, keyboard, timer};
use kernel_core::{device::pci, sync::init::InitData};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: InitData<InterruptDescriptorTable> = InitData::uninit();

macro_rules! generate_pci_handlers {
    ($table:ident, $($vec:expr => $fn_name:ident),* $(,)?) => {
        $(
            extern "x86-interrupt" fn $fn_name(_frame: InterruptStackFrame) {
                pci::PCI_HUB.get().run(|hub| hub.dispatch_interrupt($vec));
            }

            $table[$vec].set_handler_fn($fn_name);
        )*
    };
}

/// Initialize the Interrupt Descriptor Table.
///
/// # Safety
///
/// Must only be called once before any [IDT] use.
pub unsafe fn init() {
    let mut idt: InterruptDescriptorTable = InterruptDescriptorTable::new();

    idt.overflow.set_handler_fn(exceptions::overflow_handler);

    idt.debug.set_handler_fn(exceptions::debug_handler);

    idt.bound_range_exceeded
        .set_handler_fn(exceptions::bound_range_exceeded_handler);

    idt.alignment_check
        .set_handler_fn(exceptions::alignment_check_handler);

    idt.breakpoint
        .set_handler_fn(exceptions::breakpoint_handler);

    idt.cp_protection_exception
        .set_handler_fn(exceptions::cp_protection_exception_handler);

    idt.device_not_available
        .set_handler_fn(exceptions::device_not_available_handler);

    idt.divide_error
        .set_handler_fn(exceptions::divide_error_handler);

    idt.double_fault
        .set_handler_fn(exceptions::double_fault_handler);

    idt.general_protection_fault
        .set_handler_fn(exceptions::general_protection_fault_handler);

    idt.hv_injection_exception
        .set_handler_fn(exceptions::hv_injection_exception_handler);

    idt.invalid_opcode
        .set_handler_fn(exceptions::invalid_opcode_handler);

    idt.invalid_tss
        .set_handler_fn(exceptions::invalid_tss_handler);

    idt.machine_check
        .set_handler_fn(exceptions::machine_check_handler);

    idt.non_maskable_interrupt
        .set_handler_fn(exceptions::non_maskable_interrupt_handler);

    idt.page_fault
        .set_handler_fn(exceptions::page_fault_handler);

    idt.security_exception
        .set_handler_fn(exceptions::security_exception_handler);

    idt.segment_not_present
        .set_handler_fn(exceptions::segment_not_present_handler);

    idt.simd_floating_point
        .set_handler_fn(exceptions::simd_floating_point_handler);

    idt.stack_segment_fault
        .set_handler_fn(exceptions::stack_segment_fault_handler);

    idt.virtualization
        .set_handler_fn(exceptions::virtualization_exception_handler);

    idt.vmm_communication_exception
        .set_handler_fn(exceptions::vmm_communication_exception_handler);

    idt.x87_floating_point
        .set_handler_fn(exceptions::x87_floating_point_handler);

    idt[InterruptVector::Timer.with_offset()].set_handler_fn(timer::timer_handler);

    idt[InterruptVector::Keyboard.with_offset()]
        .set_handler_fn(keyboard::keyboard_interrupt_handler);

    generate_pci_handlers!(
        idt,
        34 => pci_interrupt_34_handler,
        35 => pci_interrupt_35_handler,
        36 => pci_interrupt_36_handler,
        37 => pci_interrupt_37_handler,
        38 => pci_interrupt_38_handler,
        39 => pci_interrupt_39_handler,
        40 => pci_interrupt_40_handler,
        41 => pci_interrupt_41_handler,
        42 => pci_interrupt_42_handler,
        43 => pci_interrupt_43_handler,
        44 => pci_interrupt_44_handler,
        45 => pci_interrupt_45_handler,
        46 => pci_interrupt_46_handler,
        47 => pci_interrupt_47_handler,
        48 => pci_interrupt_48_handler,
        49 => pci_interrupt_49_handler,
        50 => pci_interrupt_50_handler,
        51 => pci_interrupt_51_handler,
        52 => pci_interrupt_52_handler,
        53 => pci_interrupt_53_handler,
        54 => pci_interrupt_54_handler,
        55 => pci_interrupt_55_handler,
        56 => pci_interrupt_56_handler,
        57 => pci_interrupt_57_handler,
        58 => pci_interrupt_58_handler,
        59 => pci_interrupt_59_handler,
        60 => pci_interrupt_60_handler,
        61 => pci_interrupt_61_handler,
        62 => pci_interrupt_62_handler,
        63 => pci_interrupt_63_handler,
        64 => pci_interrupt_64_handler,
        65 => pci_interrupt_65_handler,
        66 => pci_interrupt_66_handler,
        67 => pci_interrupt_67_handler,
        68 => pci_interrupt_68_handler,
        69 => pci_interrupt_69_handler,
        70 => pci_interrupt_70_handler,
        71 => pci_interrupt_71_handler,
        72 => pci_interrupt_72_handler,
        73 => pci_interrupt_73_handler,
        74 => pci_interrupt_74_handler,
        75 => pci_interrupt_75_handler,
        76 => pci_interrupt_76_handler,
        77 => pci_interrupt_77_handler,
        78 => pci_interrupt_78_handler,
        79 => pci_interrupt_79_handler,
        80 => pci_interrupt_80_handler,
        81 => pci_interrupt_81_handler,
        82 => pci_interrupt_82_handler,
        83 => pci_interrupt_83_handler,
        84 => pci_interrupt_84_handler,
        85 => pci_interrupt_85_handler,
        86 => pci_interrupt_86_handler,
        87 => pci_interrupt_87_handler,
        88 => pci_interrupt_88_handler,
        89 => pci_interrupt_89_handler,
        90 => pci_interrupt_90_handler,
        91 => pci_interrupt_91_handler,
        92 => pci_interrupt_92_handler,
        93 => pci_interrupt_93_handler,
        94 => pci_interrupt_94_handler,
        95 => pci_interrupt_95_handler,
        96 => pci_interrupt_96_handler,
        97 => pci_interrupt_97_handler,
        98 => pci_interrupt_98_handler,
        99 => pci_interrupt_99_handler,
        100 => pci_interrupt_100_handler,
        101 => pci_interrupt_101_handler,
        102 => pci_interrupt_102_handler,
        103 => pci_interrupt_103_handler,
        104 => pci_interrupt_104_handler,
        105 => pci_interrupt_105_handler,
        106 => pci_interrupt_106_handler,
        107 => pci_interrupt_107_handler,
        108 => pci_interrupt_108_handler,
        109 => pci_interrupt_109_handler,
        110 => pci_interrupt_110_handler,
        111 => pci_interrupt_111_handler,
        112 => pci_interrupt_112_handler,
        113 => pci_interrupt_113_handler,
        114 => pci_interrupt_114_handler,
        115 => pci_interrupt_115_handler,
        116 => pci_interrupt_116_handler,
        117 => pci_interrupt_117_handler,
        118 => pci_interrupt_118_handler,
        119 => pci_interrupt_119_handler,
        120 => pci_interrupt_120_handler,
        121 => pci_interrupt_121_handler,
        122 => pci_interrupt_122_handler,
        123 => pci_interrupt_123_handler,
        124 => pci_interrupt_124_handler,
        125 => pci_interrupt_125_handler,
        126 => pci_interrupt_126_handler,
        127 => pci_interrupt_127_handler,
        128 => pci_interrupt_128_handler,
        129 => pci_interrupt_129_handler,
        130 => pci_interrupt_130_handler,
        131 => pci_interrupt_131_handler,
        132 => pci_interrupt_132_handler,
        133 => pci_interrupt_133_handler,
        134 => pci_interrupt_134_handler,
        135 => pci_interrupt_135_handler,
        136 => pci_interrupt_136_handler,
        137 => pci_interrupt_137_handler,
        138 => pci_interrupt_138_handler,
        139 => pci_interrupt_139_handler,
        140 => pci_interrupt_140_handler,
        141 => pci_interrupt_141_handler,
        142 => pci_interrupt_142_handler,
        143 => pci_interrupt_143_handler,
        144 => pci_interrupt_144_handler,
        145 => pci_interrupt_145_handler,
        146 => pci_interrupt_146_handler,
        147 => pci_interrupt_147_handler,
        148 => pci_interrupt_148_handler,
        149 => pci_interrupt_149_handler,
        150 => pci_interrupt_150_handler,
        151 => pci_interrupt_151_handler,
        152 => pci_interrupt_152_handler,
        153 => pci_interrupt_153_handler,
        154 => pci_interrupt_154_handler,
        155 => pci_interrupt_155_handler,
        156 => pci_interrupt_156_handler,
        157 => pci_interrupt_157_handler,
        158 => pci_interrupt_158_handler,
        159 => pci_interrupt_159_handler,
        160 => pci_interrupt_160_handler,
        161 => pci_interrupt_161_handler,
        162 => pci_interrupt_162_handler,
        163 => pci_interrupt_163_handler,
        164 => pci_interrupt_164_handler,
        165 => pci_interrupt_165_handler,
        166 => pci_interrupt_166_handler,
        167 => pci_interrupt_167_handler,
        168 => pci_interrupt_168_handler,
        169 => pci_interrupt_169_handler,
        170 => pci_interrupt_170_handler,
        171 => pci_interrupt_171_handler,
        172 => pci_interrupt_172_handler,
        173 => pci_interrupt_173_handler,
        174 => pci_interrupt_174_handler,
        175 => pci_interrupt_175_handler,
        176 => pci_interrupt_176_handler,
        177 => pci_interrupt_177_handler,
        178 => pci_interrupt_178_handler,
        179 => pci_interrupt_179_handler,
        180 => pci_interrupt_180_handler,
        181 => pci_interrupt_181_handler,
        182 => pci_interrupt_182_handler,
        183 => pci_interrupt_183_handler,
        184 => pci_interrupt_184_handler,
        185 => pci_interrupt_185_handler,
        186 => pci_interrupt_186_handler,
        187 => pci_interrupt_187_handler,
        188 => pci_interrupt_188_handler,
        189 => pci_interrupt_189_handler,
        190 => pci_interrupt_190_handler,
        191 => pci_interrupt_191_handler,
        192 => pci_interrupt_192_handler,
        193 => pci_interrupt_193_handler,
        194 => pci_interrupt_194_handler,
        195 => pci_interrupt_195_handler,
        196 => pci_interrupt_196_handler,
        197 => pci_interrupt_197_handler,
        198 => pci_interrupt_198_handler,
        199 => pci_interrupt_199_handler,
        200 => pci_interrupt_200_handler,
        201 => pci_interrupt_201_handler,
        202 => pci_interrupt_202_handler,
        203 => pci_interrupt_203_handler,
        204 => pci_interrupt_204_handler,
        205 => pci_interrupt_205_handler,
        206 => pci_interrupt_206_handler,
        207 => pci_interrupt_207_handler,
        208 => pci_interrupt_208_handler,
        209 => pci_interrupt_209_handler,
        210 => pci_interrupt_210_handler,
        211 => pci_interrupt_211_handler,
        212 => pci_interrupt_212_handler,
        213 => pci_interrupt_213_handler,
        214 => pci_interrupt_214_handler,
        215 => pci_interrupt_215_handler,
        216 => pci_interrupt_216_handler,
        217 => pci_interrupt_217_handler,
        218 => pci_interrupt_218_handler,
        219 => pci_interrupt_219_handler,
        220 => pci_interrupt_220_handler,
        221 => pci_interrupt_221_handler,
        222 => pci_interrupt_222_handler,
        223 => pci_interrupt_223_handler,
        224 => pci_interrupt_224_handler,
        225 => pci_interrupt_225_handler,
        226 => pci_interrupt_226_handler,
        227 => pci_interrupt_227_handler,
        228 => pci_interrupt_228_handler,
        229 => pci_interrupt_229_handler,
        230 => pci_interrupt_230_handler,
        231 => pci_interrupt_231_handler,
        232 => pci_interrupt_232_handler,
        233 => pci_interrupt_233_handler,
        234 => pci_interrupt_234_handler,
        235 => pci_interrupt_235_handler,
        236 => pci_interrupt_236_handler,
        237 => pci_interrupt_237_handler,
        238 => pci_interrupt_238_handler,
        239 => pci_interrupt_239_handler,
        240 => pci_interrupt_240_handler,
        241 => pci_interrupt_241_handler,
        242 => pci_interrupt_242_handler,
        243 => pci_interrupt_243_handler,
        244 => pci_interrupt_244_handler,
        245 => pci_interrupt_245_handler,
        246 => pci_interrupt_246_handler,
        247 => pci_interrupt_247_handler,
        248 => pci_interrupt_248_handler,
        249 => pci_interrupt_249_handler,
        250 => pci_interrupt_250_handler,
        251 => pci_interrupt_251_handler,
        252 => pci_interrupt_252_handler,
        253 => pci_interrupt_253_handler,
        254 => pci_interrupt_254_handler,
        255 => pci_interrupt_255_handler,
    );

    unsafe { IDT.init(idt).load() }
}
