// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::NStack;

use ranno::Annotation;

impl<T, A> Annotation<NStack<T, A>> for () {
    fn from_child(_: &NStack<T, A>) -> Self {}
}
