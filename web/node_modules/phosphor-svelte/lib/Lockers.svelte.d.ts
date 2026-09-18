import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LockersIcon` instead.
 *
 * @example
 * ```svelte
 * <Lockers color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Lockers: Component<IconComponentProps, {}, "">;
type Lockers = ReturnType<typeof Lockers>;
export default Lockers;
