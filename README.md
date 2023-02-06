https://www.reddit.com/r/selfhosted/comments/pr6t5l/alternatives_to_cloudinary_imgix_or_imagekit/
https://docs.imgix.com/apis/rendering
https://web.dev/image-cdns/

Under the hood, <PROJECTNAME> uses FFmpeg and ImageMagick. Shout out to these great projects!

It is meant to be used as an alternative to services like Imagix that transcode photos. Though, <PROJECTNAME> doesn't store any media, it just processes them.

<PROJECTNAME> follows the Unix philosophy.

Main priorities are to keep the API simple, ergonomic and stable whilst adding support for new big formats (supported by browsers) regularly.

I mainly use this for my other project Galera but I'd like to make a frontend (client) for it in the future when I have more time.

PRs are welcome :).