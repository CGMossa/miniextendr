// use rsys::*;
// use std::panic::{self, PanicInfo};
// use std::panic;

// panic::set_hook(Box::new(|_| {
    
//   let old_panic = panic::take_hook();
  
//   unsafe {
//     R_RunExitFinalizers();
//     CleanEd();
//     R_CleanTempDir();
//     let fatal = 0;
//     if fatal == 0 {
//       Rf_KillAllDevices();
//       AllDevicesKilled = Rboolean::TRUE;
//     }
//     //FIXME: use Rf_GetOption to gather `R_CollectWarnings` which is 00
//     GA_appcleanup();
//   }
//   old_panic.call(info)
// }));