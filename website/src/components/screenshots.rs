use leptos::prelude::*;

#[component]
pub fn Screenshots() -> impl IntoView {
    let (selected_image, set_selected_image) = signal(None::<&'static str>);

    let images = vec![
        "/public/assets/anime.png",
        "/public/assets/user.png",
        "/public/assets/character.png",
        "/public/assets/autocomplete.png",
        "/public/assets/seiyuu.png",
        "https://placehold.co/600x400/2a2438/ff6b9d?text=More+Features",
    ];

    view! {
        <section class="section screenshots" id="screenshots">
            <div class="container">
                <div class="section-title">
                    <h2>"See Kasuki in Action"</h2>
                    <p>"Take a look at Kasuki's sleek interface and powerful features."</p>
                </div>
                <div class="screenshot-gallery">
                    {images.into_iter()
                        .map(|image| {
                            view! {
                                <div class="screenshot" on:click=move |_| set_selected_image.set(Some(image))>
                                    <img src=image alt="Screenshot"/>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            {move || {
                selected_image.get().map(|image| {
                    view! {
                        <div class="image-modal active" on:click=move |_| set_selected_image.set(None)>
                            <img src=image alt="Enlarged screenshot"/>
                        </div>
                    }
                })
            }}
        </section>
    }
}