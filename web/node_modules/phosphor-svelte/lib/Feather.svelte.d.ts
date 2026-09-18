import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FeatherIcon` instead.
 *
 * @example
 * ```svelte
 * <Feather color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Feather: Component<IconComponentProps, {}, "">;
type Feather = ReturnType<typeof Feather>;
export default Feather;
