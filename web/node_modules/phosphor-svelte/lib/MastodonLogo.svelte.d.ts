import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MastodonLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MastodonLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MastodonLogo: Component<IconComponentProps, {}, "">;
type MastodonLogo = ReturnType<typeof MastodonLogo>;
export default MastodonLogo;
