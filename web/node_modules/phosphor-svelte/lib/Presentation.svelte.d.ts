import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PresentationIcon` instead.
 *
 * @example
 * ```svelte
 * <Presentation color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Presentation: Component<IconComponentProps, {}, "">;
type Presentation = ReturnType<typeof Presentation>;
export default Presentation;
