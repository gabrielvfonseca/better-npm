import { getPackageUrl, sharedUtil } from "../../../packages/shared/src/index"
console.info(sharedUtil())
console.info("Monorepo web app running!")
console.info("Package URL:", getPackageUrl("lodash", "4.17.21"))
