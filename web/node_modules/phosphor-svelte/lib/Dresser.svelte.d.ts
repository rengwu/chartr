import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DresserIcon` instead.
 *
 * @example
 * ```svelte
 * <Dresser color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Dresser: Component<IconComponentProps, {}, "">;
type Dresser = ReturnType<typeof Dresser>;
export default Dresser;
