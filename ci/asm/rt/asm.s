  .section .text.HardFaultTrampoline
  .global HardFaultTrampoline
  .thumb_func
HardFaultTrampoline:
  mrs r0, MSP
  b HardFault
