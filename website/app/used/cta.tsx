import { Button } from '@/components/shared/ui/button';

import { LandingPrimaryImageCtaSection } from '@/components/landing/cta/LandingPrimaryCta';
import { LandingProductHuntAward } from '@/components/landing/social-proof/LandingProductHuntAward';

export default function CTA() {
    return (
        <LandingPrimaryImageCtaSection
            title="Capture perfect screenshots in seconds"
            description="With a single API call, you can let your screenshot dreams fly, leaving the grunt work to us."
            imageSrc="/public/img/anime.png"
            imageAlt="Sample image"
            withBackground
            leadingComponent={<LandingProductHuntAward />}
        >
            <Button size="xl" asChild>
                <a href="#">Sign up</a>
            </Button>

            <Button size="xl" variant="outlinePrimary">
                <a href="#">See demo</a>
            </Button>
        </LandingPrimaryImageCtaSection>
    );
}
