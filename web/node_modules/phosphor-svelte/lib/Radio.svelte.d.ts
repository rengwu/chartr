import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RadioIcon` instead.
 *
 * @example
 * ```svelte
 * <Radio color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Radio: Component<IconComponentProps, {}, "">;
type Radio = ReturnType<typeof Radio>;
export default Radio;
