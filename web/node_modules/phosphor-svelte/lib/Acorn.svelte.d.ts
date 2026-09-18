import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AcornIcon` instead.
 *
 * @example
 * ```svelte
 * <Acorn color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Acorn: Component<IconComponentProps, {}, "">;
type Acorn = ReturnType<typeof Acorn>;
export default Acorn;
