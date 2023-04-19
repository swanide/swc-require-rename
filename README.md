@swanide/swc-require-rename
====

swc plugin to add module prefix.

__@swc/core: ">=1.3.42"__

# config

```javascript
[
    "@swanide/swc-require-rename",
    {
        "modulePrefix": "@swan-module/"
    }
]
```

source:

```javascript
import fs from 'fs';
import {resolve} from 'path';
import {debounce} from 'lodash';
const a = require('../abc');
require('/def');
require('./../def');
export * from 'exp1';
export {ee} from 'exp1';
```

transformed:

```javascript
"use strict";
import fs from '@swan-module/fs';
import {resolve} from '@swan-module/path';
import {debounce} from '@swan-module/lodash';
const a = require('@swan-module/../abc');
require('@swan-module//def');
require('@swan-module/./../def');
export * from '@swan-module/exp1';
export {ee} from '@swan-module/exp1';
```

