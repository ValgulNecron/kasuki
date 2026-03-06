use leptos::prelude::*;

#[component]
pub fn Screenshots() -> impl IntoView {
    let (selected_image, set_selected_image) = signal(None::<(&'static str, &'static str)>);

    let images: Vec<(&'static str, &'static str)> = vec![
        ("/public/assets/anime.webp", "Kasuki anime search command showing detailed anime information"),
        ("/public/assets/user.webp", "Kasuki user profile command displaying AniList statistics"),
        ("/public/assets/character.webp", "Kasuki character search showing anime character details"),
        ("/public/assets/autocomplete.webp", "Kasuki autocomplete feature for easy command usage"),
        ("/public/assets/seiyuu.webp", "Kasuki seiyuu command showing voice actor information"),
        ("https://placehold.co/600x400/2a2438/ff6b9d?text=More+Features", "More Kasuki features coming soon"),
    ];

    view! {
        <section class="section screenshots" id="screenshots" aria-labelledby="screenshots-title">
            <div class="container">
                <div class="section-title">
                    <h2 id="screenshots-title">"See Kasuki in Action"</h2>
                    <p>"Take a look at Kasuki's sleek interface and powerful features."</p>
                </div>
                <div class="screenshot-gallery" role="list">
                    {images.into_iter()
                        .map(|(image, alt)| {
                            view! {
                                <div class="screenshot" role="listitem" on:click=move |_| set_selected_image.set(Some((image, alt)))>
                                    <img 
                                        src=image 
                                        alt=alt
                                        loading="lazy"
                                        width="600"
                                        height="400"
                                        decoding="async"
                                    />
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            {move || {
                selected_image.get().map(|(image, alt)| {
                    view! {
                        <div 
                            class="image-modal active" 
                            on:click=move |_| set_selected_image.set(None)
                            role="dialog"
                            aria-label="Enlarged screenshot view"
                        >
                            <img src=image alt=alt/>
                        </div>
                    }
                })
            }}
        </section>
    }
}