import test from 'ava'

import { getVersion } from '../index'

test('sync function from native code', (t) => {
  t.is(getVersion().major, 1)
})
