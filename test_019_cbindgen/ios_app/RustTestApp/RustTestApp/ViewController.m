//
//  ViewController.m
//  RustTestApp
//
//  Created by DevNul on 14.01.2020.
//  Copyright © 2020 DevNul. All rights reserved.
//

#import "ViewController.h"
#include <library.h>

@interface ViewController ()
@end

@implementation ViewController

- (void)viewDidLoad {
    [super viewDidLoad];
    // Do any additional setup after loading the view, typically from a nib.
    
    // https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-06-rust-on-ios.html
    // При линковке с дебажной библиотекой - отладчик без проблем заходит внутрь функций
    int32_t res = function_1(5);
    NSLog(@"Value from rust: %d", res);
}


@end
