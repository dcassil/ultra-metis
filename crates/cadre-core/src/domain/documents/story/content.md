# {{ title }}
{% if parent_title is defined and parent_title != "" %}

**Epic**: {{ parent_title }}{% if parent_short_code is defined and parent_short_code != "" %} ({{ parent_short_code }}){% endif %}
{% endif %}

## Objective

<!-- What this story achieves and why it matters within its Epic. -->

## Scope

<!-- What is included and explicitly excluded from this story. -->
**In Scope**:
- (what this story covers)

**Out of Scope**:
- (what this story does NOT cover)

## Approach

<!-- Implementation strategy and key technical decisions. -->

## Design References

<!-- Links to relevant DesignContext documents or design artifacts. -->

## Validation Expectations

<!-- How we verify this story is complete and correct. Be specific about what tests or checks must pass. -->