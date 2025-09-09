// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: MIT

use slint_build::CompileError;


fn main() -> Result<(), CompileError> {
    slint_build::compile("ui/fifteen.slint")
}
