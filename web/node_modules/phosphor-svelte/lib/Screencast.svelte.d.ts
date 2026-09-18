import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ScreencastIcon` instead.
 *
 * @example
 * ```svelte
 * <Screencast color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Screencast: Component<IconComponentProps, {}, "">;
type Screencast = ReturnType<typeof Screencast>;
export default Screencast;
