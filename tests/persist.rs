// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#[cfg(feature = "persistance")]
mod persist {

    use microkelvin::{BackendCtor, Compound, DiskBackend, Persistance};
    use nstack::NStack;

    #[test]
    fn persist_across_threads() {
        let n: u64 = 1024;

        let mut stack = NStack::<u64, ()>::new();

        for i in 0..n {
            stack.push(i).unwrap();
        }

        let backend = BackendCtor::new(|| DiskBackend::ephemeral().unwrap());
        let persisted = Persistance::persist(&backend, &stack).unwrap();

        // it should now be available from other threads

        std::thread::spawn(move || {
            let restored_generic = persisted.restore().unwrap();

            let mut restored: NStack<u64, ()> =
                NStack::from_generic(&restored_generic).unwrap();

            for i in 0..n {
                assert_eq!(restored.pop().unwrap(), Some(n - i - 1));
            }
        })
        .join()
        .unwrap();

        // then empty the original

        for i in 0..n {
            assert_eq!(stack.pop().unwrap(), Some(n - i - 1));
        }
    }
}
