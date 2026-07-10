import { sharedUtil, getPackageUrl } from '@dash/example-shared';

console.log(sharedUtil());
console.log('Monorepo web app running!');
console.log('Package URL:', getPackageUrl('lodash', '4.17.21'));
