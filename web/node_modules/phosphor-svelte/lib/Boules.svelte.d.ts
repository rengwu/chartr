import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BoulesIcon` instead.
 *
 * @example
 * ```svelte
 * <Boules color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Boules: Component<IconComponentProps, {}, "">;
type Boules = ReturnType<typeof Boules>;
export default Boules;
