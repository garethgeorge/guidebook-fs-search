# Guidebook

A web based file browser for your NAS offering full text search built on [tantivy](https://github.com/quickwit-oss/tantivy). Guidebook is quick to deploy and ships as a single binary with no external dependencies.

Guidebook is designed from first principles to be extremely light weight, blazingly fast, and quick to deploy.

# Known Limitations

 * Index size: guidebook's index is append only and will slowly grow over time without bound. 
 * Only new files will be reindexed. 
 * It is recommended to install a cron to reset the index and re-map your filesystem at an interval that suits your deployment. 

# Dependencies
 - rust
 - cargo

# Roadmap
 - [ ] Document Indexing
    - [ ] Basic text indexing
    - [ ] PDF Support
    - [ ] Tesseract support for OCR
 - [ ] Incremental Indexing - add files to transactions in batches
 - [ ] Search
   - [ ] Basic queries via CLI
   - [ ] Web frontend
   - [ ] Image previews of supported document types
 - [ ] Filesystems
   - [ ] Multi-filesystem support
   - [ ] S3 support
   - [ ] SSHFS support
