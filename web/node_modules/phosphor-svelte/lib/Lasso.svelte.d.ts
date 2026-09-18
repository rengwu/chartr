import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LassoIcon` instead.
 *
 * @example
 * ```svelte
 * <Lasso color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Lasso: Component<IconComponentProps, {}, "">;
type Lasso = ReturnType<typeof Lasso>;
export default Lasso;
