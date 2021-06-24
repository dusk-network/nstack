// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#[cfg(feature = "persistance")]
mod persist {

    use microkelvin::{
        BackendCtor, Compound, DiskBackend, PersistError, Persistance,
    };
    use nstack::NStack;

    #[test]
    fn persist_across_threads() -> Result<(), PersistError> {
        let n: u64 = 1024;

        let mut stack = NStack::<u64, ()>::new();

        for i in 0..n {
            stack.push(i)?;
        }

        let backend = BackendCtor::new(|| DiskBackend::ephemeral());
        let persisted = Persistance::persist(&backend, &stack)?;

        // it should now be available from other threads

        std::thread::spawn(move || {
            let restored_generic = persisted.restore()?;

            let mut restored: NStack<u64, ()> =
                NStack::from_generic(&restored_generic)?;

            for i in 0..n {
                assert_eq!(restored.pop()?, Some(n - i - 1));
            }
            Ok(()) as Result<(), PersistError>
        })
        .join()
        .expect("thread to join cleanly")?;

        // then empty the original

        for i in 0..n {
            assert_eq!(stack.pop()?, Some(n - i - 1));
        }
        Ok(())
    }
}
