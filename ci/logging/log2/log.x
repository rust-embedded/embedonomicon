SECTIONS
{
  .log 0 (INFO) : {
    *(.log.error);
    __log_warning_start__ = .;
    *(.log.warning);
  }
}
